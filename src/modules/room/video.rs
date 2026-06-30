use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use axum::extract::Multipart;
use mongodb::{
    Collection,
    bson::{doc, oid::ObjectId},
    options::ReturnDocument,
};
use tokio::{
    fs,
    io::AsyncWriteExt,
    process::Command,
};
use uuid::Uuid;

use crate::common::error::AppError;

use super::{
    dto::RoomResponse,
    model::{Room, VideoStatus},
};

const MAX_VIDEO_SIZE: u64 = 2 * 1024 * 1024 * 1024;

pub async fn upload_video_for_room(
    room_collection: Collection<Room>,
    room_id: ObjectId,
    mut multipart: Multipart,
) -> Result<RoomResponse, AppError> {
    ensure_room_exists(&room_collection, room_id).await?;

    let video_id = Uuid::new_v4().to_string();

    let upload_directory = std::env::var("UPLOAD_DIRECTORY")
        .unwrap_or_else(|_| "./media/uploads".to_string());

    let hls_directory = std::env::var("HLS_DIRECTORY")
        .unwrap_or_else(|_| "./media/hls".to_string());

    let public_hls_url = std::env::var("PUBLIC_HLS_URL")
        .unwrap_or_else(|_| {
            "http://localhost:8080/media".to_string()
        });

    fs::create_dir_all(&upload_directory)
        .await
        .map_err(|error| {
            eprintln!("Failed to create upload directory: {error}");

            AppError::internal(
                "Failed to create upload directory",
            )
        })?;

    let video_hls_directory =
        Path::new(&hls_directory).join(&video_id);

    fs::create_dir_all(&video_hls_directory)
        .await
        .map_err(|error| {
            eprintln!("Failed to create HLS directory: {error}");

            AppError::internal(
                "Failed to create HLS directory",
            )
        })?;

    let upload_path = save_uploaded_video(
        &mut multipart,
        Path::new(&upload_directory),
        &video_id,
    )
    .await?;

    convert_video_to_hls(
        &upload_path,
        &video_hls_directory,
    )
    .await?;

    let video_url = format!(
        "{}/{}/index.m3u8",
        public_hls_url.trim_end_matches('/'),
        video_id,
    );

    let updated_room = update_room_video(
        room_collection,
        room_id,
        &video_url,
    )
    .await?;

    // Facultatif : supprimer le fichier source après conversion.
    if let Err(error) = fs::remove_file(&upload_path).await {
        eprintln!(
            "Failed to remove uploaded source file {}: {error}",
            upload_path.display(),
        );
    }

    Ok(RoomResponse::from(updated_room))
}

async fn ensure_room_exists(
    room_collection: &Collection<Room>,
    room_id: ObjectId,
) -> Result<(), AppError> {
    let exists = room_collection
        .find_one(doc! {
            "_id": room_id,
        })
        .await?
        .is_some();

    if !exists {
        return Err(AppError::not_found("Room not found"));
    }

    Ok(())
}

async fn save_uploaded_video(
    multipart: &mut Multipart,
    upload_directory: &Path,
    video_id: &str,
) -> Result<PathBuf, AppError> {
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|error| {
            eprintln!("Invalid multipart request: {error}");

            AppError::bad_request(
                "Invalid multipart request",
            )
        })?
    {
        if field.name() != Some("video") {
            continue;
        }

        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        if !is_supported_video_type(&content_type) {
            return Err(AppError::bad_request(
                "Unsupported video format",
            ));
        }

        let extension =
            extension_from_content_type(&content_type);

        let upload_path = upload_directory.join(
            format!("{video_id}.{extension}"),
        );

        let mut file = fs::File::create(&upload_path)
            .await
            .map_err(|error| {
                eprintln!(
                    "Failed to create upload file: {error}"
                );

                AppError::internal(
                    "Failed to save uploaded video",
                )
            })?;

        let mut total_size = 0_u64;

        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|error| {
                eprintln!(
                    "Failed to read upload chunk: {error}"
                );

                AppError::bad_request(
                    "Failed to read uploaded video",
                )
            })?
        {
            total_size += chunk.len() as u64;

            if total_size > MAX_VIDEO_SIZE {
                let _ = fs::remove_file(&upload_path).await;

                return Err(AppError::bad_request(
                    "Uploaded video is too large",
                ));
            }

            file.write_all(&chunk)
                .await
                .map_err(|error| {
                    eprintln!(
                        "Failed to write upload chunk: {error}"
                    );

                    AppError::internal(
                        "Failed to save uploaded video",
                    )
                })?;
        }

        if total_size == 0 {
            let _ = fs::remove_file(&upload_path).await;

            return Err(AppError::bad_request(
                "Uploaded video is empty",
            ));
        }

        file.flush()
            .await
            .map_err(|error| {
                eprintln!(
                    "Failed to flush upload file: {error}"
                );

                AppError::internal(
                    "Failed to save uploaded video",
                )
            })?;

        return Ok(upload_path);
    }

    Err(AppError::bad_request(
        "Missing multipart field 'video'",
    ))
}

async fn convert_video_to_hls(
    input_path: &Path,
    output_directory: &Path,
) -> Result<(), AppError> {
    let playlist_path =
        output_directory.join("index.m3u8");

    let segment_path =
        output_directory.join("segment_%05d.ts");

    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg("fast")
        .arg("-crf")
        .arg("23")
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg("128k")
        .arg("-force_key_frames")
        .arg("expr:gte(t,n_forced*6)")
        .arg("-hls_time")
        .arg("6")
        .arg("-hls_playlist_type")
        .arg("vod")
        .arg("-hls_segment_filename")
        .arg(&segment_path)
        .arg(&playlist_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|error| {
            eprintln!("Failed to start FFmpeg: {error}");

            AppError::internal(
                "Failed to start video conversion",
            )
        })?;

    if !output.status.success() {
        let stderr =
            String::from_utf8_lossy(&output.stderr);

        eprintln!("FFmpeg conversion failed:\n{stderr}");

        let _ = fs::remove_dir_all(output_directory).await;

        return Err(AppError::internal(
            "Video conversion failed",
        ));
    }

    Ok(())
}

async fn update_room_video(
    room_collection: Collection<Room>,
    room_id: ObjectId,
    video_url: &str,
) -> Result<Room, AppError> {
    let timestamp = unix_timestamp()?;

    room_collection
        .find_one_and_update(
            doc! {
                "_id": room_id,
            },
            doc! {
                "$set": {
                    "video_url": video_url,
                    "video_status": serialize_video_status(
                        VideoStatus::Paused,
                    )?,
                    "position_seconds": 0.0,
                    "updated_at": timestamp,
                },
            },
        )
        .return_document(ReturnDocument::After)
        .await?
        .ok_or_else(|| {
            AppError::not_found("Room not found")
        })
}

fn unix_timestamp() -> Result<i64, AppError> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| {
            AppError::internal("Invalid system time")
        })?
        .as_secs();

    i64::try_from(seconds).map_err(|_| {
        AppError::internal("Timestamp is too large")
    })
}

fn serialize_video_status(
    status: VideoStatus,
) -> Result<mongodb::bson::Bson, AppError> {
    mongodb::bson::to_bson(&status).map_err(|error| {
        eprintln!(
            "Failed to serialize video status: {error}"
        );

        AppError::internal(
            "Failed to serialize video status",
        )
    })
}

fn is_supported_video_type(content_type: &str) -> bool {
    matches!(
        content_type,
        "video/mp4"
            | "video/webm"
            | "video/quicktime"
            | "video/x-matroska"
            | "application/octet-stream"
    )
}

fn extension_from_content_type(
    content_type: &str,
) -> &'static str {
    match content_type {
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        "video/x-matroska" => "mkv",
        _ => "mp4",
    }
}
