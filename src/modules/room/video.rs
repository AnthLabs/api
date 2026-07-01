use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use axum::extract::Multipart;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use mongodb::{
    Collection,
    bson::{doc, oid::ObjectId},
    options::ReturnDocument,
};
use serde_json::json;
use sha2::Sha256;
use tokio::{
    fs,
    io::AsyncWriteExt,
    process::Command,
};
use uuid::Uuid;

use crate::common::{
    error::AppError,
    media::{hls_directory, keys_directory, uploads_directory},
};

use super::{
    dto::RoomResponse,
    model::{Room, VideoStatus},
    websocket::{message::ServerMessage, hub::RoomHub},
};

const MAX_VIDEO_SIZE: u64 = 2 * 1024 * 1024 * 1024;
type HmacSha256 = Hmac<Sha256>;

pub async fn upload_video_for_room(
    room_collection: Collection<Room>,
    room_hub: RoomHub,
    room_id: ObjectId,
    mut multipart: Multipart,
) -> Result<RoomResponse, AppError> {
    ensure_room_exists(&room_collection, room_id).await?;

    let video_id = Uuid::new_v4().to_string();

    let uploads_path = uploads_directory();
    let hls_path = hls_directory();
    let keys_path = keys_directory();
    let public_media_url = public_media_url();

    fs::create_dir_all(&uploads_path)
        .await
        .map_err(|error| {
            eprintln!("Failed to create upload directory: {error}");

            AppError::internal(
                "Failed to create upload directory",
            )
        })?;

    fs::create_dir_all(&keys_path)
        .await
        .map_err(|error| {
            eprintln!("Failed to create key directory: {error}");

            AppError::internal(
                "Failed to create key directory",
            )
        })?;

    let video_hls_directory = hls_path.join(&video_id);

    fs::create_dir_all(&video_hls_directory)
        .await
        .map_err(|error| {
            eprintln!("Failed to create HLS directory: {error}");

            AppError::internal(
                "Failed to create HLS directory",
            )
        })?;

    let upload_path = save_uploaded_video(&mut multipart, &uploads_path, &video_id).await?;

    if let Err(error) =
        convert_video_to_hls(&upload_path, &video_hls_directory, &keys_path, &video_id).await
    {
        let _ = fs::remove_file(&upload_path).await;
        let _ = fs::remove_dir_all(&video_hls_directory).await;
        let _ = fs::remove_file(keys_path.join(format!("{video_id}.key"))).await;

        return Err(error);
    }

    let video_url = format!(
        "{}/hls/{}/master.m3u8",
        public_media_url.trim_end_matches('/'),
        video_id,
    );

    let updated_room = update_room_video(room_collection, room_id, &video_url).await?;

    let room_response = RoomResponse::from(updated_room);

    room_hub
        .broadcast(
            room_id,
            ServerMessage::RoomUpdated {
                room: room_response.clone(),
            },
        )
        .await;

    Ok(room_response)
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
            total_size = total_size
                .checked_add(chunk.len() as u64)
                .ok_or_else(|| {
                    AppError::bad_request(
                        "Uploaded video is too large",
                    )
                })?;

            if total_size > MAX_VIDEO_SIZE {
                drop(file);

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
            drop(file);

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
    key_directory: &Path,
    asset_id: &str,
) -> Result<(), AppError> {
    let playlist_path =
        output_directory.join("master.m3u8");

    let segment_path =
        output_directory.join("segment_%05d.ts");

    let key_file_path =
        key_directory.join(format!("{asset_id}.key"));

    let key_info_path =
        key_directory.join(format!("{asset_id}.keyinfo.tmp"));

    let aes_key = random_bytes();
    let iv = hex_encode(&random_bytes());
    let token = create_key_token(asset_id)?;
    let key_uri = format!(
        "{}/keys/{}.key?token={}",
        key_server_public_url().trim_end_matches('/'),
        asset_id,
        token,
    );

    fs::write(&key_file_path, &aes_key)
        .await
        .map_err(|error| {
            eprintln!("Failed to write HLS key: {error}");

            AppError::internal("Failed to write HLS key")
        })?;

    let key_info = format!(
        "{key_uri}\n{}\n{iv}\n",
        key_file_path.display(),
    );

    fs::write(&key_info_path, key_info)
        .await
        .map_err(|error| {
            eprintln!("Failed to write FFmpeg key info file: {error}");

            let _ = std::fs::remove_file(&key_file_path);

            AppError::internal("Failed to prepare encrypted HLS conversion")
        })?;

    let output_result = Command::new("ffmpeg")
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
        .arg("-hls_key_info_file")
        .arg(&key_info_path)
        .arg("-hls_segment_filename")
        .arg(&segment_path)
        .arg(&playlist_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await;

    let _ = fs::remove_file(&key_info_path).await;

    let output = match output_result {
        Ok(output) => output,
        Err(error) => {
            eprintln!("Failed to start FFmpeg: {error}");

            let _ = fs::remove_file(&key_file_path).await;

            return Err(AppError::internal(
                "Failed to start video conversion",
            ));
        }
    };

    if !output.status.success() {
        let stderr =
            String::from_utf8_lossy(&output.stderr);

        eprintln!("FFmpeg conversion failed:\n{stderr}");

        let _ =
            fs::remove_dir_all(output_directory).await;
        let _ = fs::remove_file(&key_file_path).await;

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

fn public_media_url() -> String {
    std::env::var("PUBLIC_MEDIA_URL")
        .unwrap_or_else(|_| {
            "http://localhost:8080/media".to_string()
        })
}

fn key_server_public_url() -> String {
    std::env::var("KEY_SERVER_PUBLIC_URL")
        .unwrap_or_else(|_| "http://localhost:8090".to_string())
}

fn key_token_secret() -> String {
    std::env::var("KEY_TOKEN_SECRET")
        .unwrap_or_else(|_| "replace-with-a-long-random-local-secret".to_string())
}

fn key_token_ttl_seconds() -> i64 {
    std::env::var("KEY_TOKEN_TTL_SECONDS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|ttl| *ttl > 0)
        .unwrap_or(3600)
}

fn create_key_token(asset_id: &str) -> Result<String, AppError> {
    let exp = unix_timestamp()?
        .checked_add(key_token_ttl_seconds())
        .ok_or_else(|| AppError::internal("Token expiration is too large"))?;

    let payload = json!({
        "asset": asset_id,
        "exp": exp,
    })
    .to_string();

    let payload_part = URL_SAFE_NO_PAD.encode(payload.as_bytes());
    let mut mac = HmacSha256::new_from_slice(key_token_secret().as_bytes())
        .map_err(|_| AppError::internal("Invalid key token secret"))?;

    mac.update(payload_part.as_bytes());

    let signature = mac.finalize().into_bytes();

    Ok(format!(
        "{}.{}",
        payload_part,
        URL_SAFE_NO_PAD.encode(signature),
    ))
}

fn random_bytes() -> [u8; 16] {
    rand::random::<[u8; 16]>()
}

fn hex_encode(bytes: &[u8; 16]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
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
