use crate::common::error::AppError;

use mongodb::{
    Collection,
    bson::{Document, doc, oid::ObjectId},
    options::ReturnDocument,
};

use super::{
    dto::{
        RoomDeleteResponse,
        RoomListResponse,
        RoomResponse,
    },
    model::Room,
};

pub async fn get_all_room(
    room_collection: Collection<Room>,
) -> Result<RoomListResponse, AppError> {
    let mut cursor = room_collection.find(doc! {}).await?;
    let mut results = Vec::new();

    while cursor.advance().await? {
        let room = cursor.deserialize_current()?;
        results.push(room);
    }

    Ok(RoomListResponse::from(results))
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

    Ok(RoomResponse::from(
        room,
    ))
}

pub async fn create_room(
    room_collection: Collection<Room>,
    room: Room,
) -> Result<RoomResponse, AppError> {
    let id = room.id;

    room_collection
        .insert_one(&room)
        .await?;

    let created_room = room_collection
        .find_one(doc! {
            "_id": id,
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
