use crate::common::error::AppError;
use std::time::{SystemTime, UNIX_EPOCH};

use mongodb::{
    Collection,
    bson::{Bson, doc, oid::ObjectId, to_bson},
    options::ReturnDocument,
};

use super::{
    dto::{RoomDeleteResponse, RoomResponse},
    model::{Room, VideoStatus},
};

#[derive(Debug)]
pub enum PlaybackCommand {
    ChangeVideo { video_url: String },
    Play { position_seconds: f64 },
    Pause { position_seconds: f64 },
    Seek { position_seconds: f64 },
}

pub async fn apply_playback_command(
    room_collection: Collection<Room>,
    id: ObjectId,
    command: PlaybackCommand,
) -> Result<RoomResponse, AppError> {
    let timestamp = unix_timestamp()?;

    let timestamp =
        i64::try_from(timestamp).map_err(|_| AppError::internal("Timestamp is too large"))?;

    let mut update = doc! {
        "updated_at": timestamp,
    };

    match command {
        PlaybackCommand::ChangeVideo { video_url } => {
            let video_url = video_url.trim();

            if video_url.is_empty() {
                return Err(AppError::bad_request("Video URL cannot be empty"));
            }

            update.insert("video_url", video_url);

            update.insert("video_status", serialize_video_status(VideoStatus::Paused)?);

            update.insert("position_seconds", 0.0);
        }

        PlaybackCommand::Play { position_seconds } => {
            validate_position(position_seconds)?;

            update.insert(
                "video_status",
                serialize_video_status(VideoStatus::Playing)?,
            );

            update.insert("position_seconds", position_seconds);
        }

        PlaybackCommand::Pause { position_seconds } => {
            validate_position(position_seconds)?;

            update.insert("video_status", serialize_video_status(VideoStatus::Paused)?);

            update.insert("position_seconds", position_seconds);
        }

        PlaybackCommand::Seek { position_seconds } => {
            validate_position(position_seconds)?;

            update.insert("position_seconds", position_seconds);
        }
    }

    let room = room_collection
        .find_one_and_update(
            doc! {
                "_id": id,
            },
            doc! {
                "$set": update,
            },
        )
        .return_document(ReturnDocument::After)
        .await?
        .ok_or_else(|| AppError::not_found("Room not found"))?;

    Ok(RoomResponse::from(room))
}

fn validate_position(position_seconds: f64) -> Result<(), AppError> {
    if !position_seconds.is_finite() || position_seconds < 0.0 {
        return Err(AppError::bad_request("Invalid video position"));
    }

    Ok(())
}

fn unix_timestamp() -> Result<u64, AppError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| AppError::internal("System time is before UNIX_EPOCH"))
}

fn serialize_video_status(status: VideoStatus) -> Result<Bson, AppError> {
    to_bson(&status).map_err(|_| AppError::internal("Failed to serialize video status"))
}
pub async fn get_room(
    room_collection: Collection<Room>,
    id: ObjectId,
) -> Result<RoomResponse, AppError> {
    let room = room_collection
        .find_one(doc! {
            "_id": id,
        })
        .await?
        .ok_or_else(|| AppError::not_found("Room not found"))?;

    Ok(RoomResponse::from(room))
}

pub async fn create_room(room_collection: Collection<Room>) -> Result<RoomResponse, AppError> {
    let id_room = ObjectId::new();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward")
        .as_secs();
    let room = Room {
        id: id_room,
        video_url: None,
        video_status: VideoStatus::Paused,
        position_seconds: 0.0,
        created_at: timestamp,
        updated_at: None,
    };

    room_collection.insert_one(&room).await?;

    let created_room = room_collection
        .find_one(doc! {
            "_id": id_room,
        })
        .await?
        .ok_or_else(|| AppError::not_found("Room not found after creation"))?;

    Ok(RoomResponse::from(created_room))
}

pub async fn delete_room(
    room_collection: Collection<Room>,
    id: ObjectId,
) -> Result<RoomDeleteResponse, AppError> {
    let result = room_collection
        .delete_one(doc! {
            "_id": id,
        })
        .await?;

    if result.deleted_count == 0 {
        return Err(AppError::not_found("Room not found"));
    }

    Ok(RoomDeleteResponse { id: id.to_hex() })
}
