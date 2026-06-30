use axum::{
    Json,
    extract::{Path, State},
};
use mongodb::bson::{Document, oid::ObjectId};

use crate::{
    common::{
        error::AppError,
        response::{ApiResponse, AppResult},
    },
    state::AppState,
};

use super::{
    dto::{
        RoomDeleteResponse,
        RoomResponse,
    },
    model::Room,
    service,
};

pub async fn get_room_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<RoomResponse> {
    let id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request("Invalid room id"))?;

    let room_collection = state
        .database
        .collection::<Room>("room");

    let response =
        service::get_room(room_collection, id).await?;

    Ok(ApiResponse::success(response))
}

pub async fn create_room_handler(
    State(state): State<AppState>,
) -> AppResult<RoomResponse> {
    let room_collection = state
        .database
        .collection::<Room>("room");

    let response = service::create_room(
        room_collection,
    )
    .await?;

    Ok(ApiResponse::success(response))
}

pub async fn update_room_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(update): Json<Document>,
) -> AppResult<RoomResponse> {
    let id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request("Invalid room id"))?;

    let room_collection = state
        .database
        .collection::<Room>("room");

    let response = service::update_room(
        room_collection,
        id,
        update,
    )
    .await?;

    Ok(ApiResponse::success(response))
}

pub async fn delete_room_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<RoomDeleteResponse> {
    let id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request("Invalid room id"))?;

    let room_collection = state
        .database
        .collection::<Room>("room");

    let response = service::delete_room(
        room_collection,
        id,
    )
    .await?;

    Ok(ApiResponse::success(response))
}
