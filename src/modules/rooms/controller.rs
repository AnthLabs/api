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
        RoomsDeleteResponse,
        RoomsListResponse,
        RoomsResponse,
    },
    model::Rooms,
    service,
};

pub async fn get_all_rooms_handler(
    State(state): State<AppState>,
) -> AppResult<RoomsListResponse> {
    let rooms_collection = state
        .database
        .collection::<Rooms>("rooms");

    let response =
        service::get_all_rooms(rooms_collection).await?;

    Ok(ApiResponse::success(response))
}

pub async fn get_rooms_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<RoomsResponse> {
    let id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request("Invalid rooms id"))?;

    let rooms_collection = state
        .database
        .collection::<Rooms>("rooms");

    let response =
        service::get_rooms(rooms_collection, id).await?;

    Ok(ApiResponse::success(response))
}

pub async fn create_rooms_handler(
    State(state): State<AppState>,
    Json(rooms): Json<Rooms>,
) -> AppResult<RoomsResponse> {
    let rooms_collection = state
        .database
        .collection::<Rooms>("rooms");

    let response = service::create_rooms(
        rooms_collection,
        rooms,
    )
    .await?;

    Ok(ApiResponse::success(response))
}

pub async fn update_rooms_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(update): Json<Document>,
) -> AppResult<RoomsResponse> {
    let id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request("Invalid rooms id"))?;

    let rooms_collection = state
        .database
        .collection::<Rooms>("rooms");

    let response = service::update_rooms(
        rooms_collection,
        id,
        update,
    )
    .await?;

    Ok(ApiResponse::success(response))
}

pub async fn delete_rooms_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<RoomsDeleteResponse> {
    let id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request("Invalid rooms id"))?;

    let rooms_collection = state
        .database
        .collection::<Rooms>("rooms");

    let response = service::delete_rooms(
        rooms_collection,
        id,
    )
    .await?;

    Ok(ApiResponse::success(response))
}
