use crate::common::error::AppError;
use std::time::{SystemTime,UNIX_EPOCH};

use mongodb::{
    Collection,
    bson::{Document, doc, oid::ObjectId},
    options::ReturnDocument,
};

use super::{
    dto::{
        RoomDeleteResponse,
        RoomResponse,
    },
    model::{Room, VideoStatus},
};

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

    Ok(RoomResponse::from(
        room,
    ))
}

pub async fn create_room(
    room_collection: Collection<Room>,
) -> Result<RoomResponse, AppError> {
    let id_room = ObjectId::new();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("time should go forward").as_secs();
    let room = Room {
        id: id_room,
        video_url: None,
        video_status: VideoStatus::Paused,
        position_second: 0,
        created_at: timestamp 
    };

    room_collection
        .insert_one(&room)
        .await?;

    let created_room = room_collection
        .find_one(doc! {
            "_id": id_room,
        })
        .await?
        .ok_or_else(|| {
            AppError::not_found(
                "Room not found after creation",
            )
        })?;

    Ok(RoomResponse::from(
        created_room,
    ))
}

pub async fn update_room(
    room_collection: Collection<Room>,
    id: ObjectId,
    update: Document,
) -> Result<RoomResponse, AppError> {
    let updated_room = room_collection
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

    Ok(RoomResponse::from(
        updated_room,
    ))
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
        return Err(AppError::not_found(
            "Room not found",
        ));
    }

    Ok(RoomDeleteResponse {
        id: id.to_hex(),
    })
}
