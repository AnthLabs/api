use crate::common::error::AppError;

use mongodb::{
    Collection,
    bson::{Document, doc, oid::ObjectId},
    options::ReturnDocument,
};

use super::{
    dto::{
        RoomsDeleteResponse,
        RoomsListResponse,
        RoomsResponse,
    },
    model::Rooms,
};

pub async fn get_all_rooms(
    rooms_collection: Collection<Rooms>,
) -> Result<RoomsListResponse, AppError> {
    let mut cursor = rooms_collection.find(doc! {}).await?;
    let mut results = Vec::new();

    while cursor.advance().await? {
        let rooms = cursor.deserialize_current()?;
        results.push(rooms);
    }

    Ok(RoomsListResponse::from(results))
}

pub async fn get_rooms(
    rooms_collection: Collection<Rooms>,
    id: ObjectId,
) -> Result<RoomsResponse, AppError> {
    let rooms = rooms_collection
        .find_one(doc! {
            "_id": id,
        })
        .await?
        .ok_or_else(|| AppError::not_found("Rooms not found"))?;

    Ok(RoomsResponse::from(
        rooms,
    ))
}

pub async fn create_rooms(
    rooms_collection: Collection<Rooms>,
    rooms: Rooms,
) -> Result<RoomsResponse, AppError> {
    let id = rooms.id;

    rooms_collection
        .insert_one(&rooms)
        .await?;

    let created_rooms = rooms_collection
        .find_one(doc! {
            "_id": id,
        })
        .await?
        .ok_or_else(|| {
            AppError::not_found(
                "Rooms not found after creation",
            )
        })?;

    Ok(RoomsResponse::from(
        created_rooms,
    ))
}

pub async fn update_rooms(
    rooms_collection: Collection<Rooms>,
    id: ObjectId,
    update: Document,
) -> Result<RoomsResponse, AppError> {
    let updated_rooms = rooms_collection
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
        .ok_or_else(|| AppError::not_found("Rooms not found"))?;

    Ok(RoomsResponse::from(
        updated_rooms,
    ))
}

pub async fn delete_rooms(
    rooms_collection: Collection<Rooms>,
    id: ObjectId,
) -> Result<RoomsDeleteResponse, AppError> {
    let result = rooms_collection
        .delete_one(doc! {
            "_id": id,
        })
        .await?;

    if result.deleted_count == 0 {
        return Err(AppError::not_found(
            "Rooms not found",
        ));
    }

    Ok(RoomsDeleteResponse {
        id: id.to_hex(),
    })
}
