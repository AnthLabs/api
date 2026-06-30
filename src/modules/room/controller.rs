use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Path, State, Multipart};
use mongodb::bson::oid::ObjectId;
use axum::Json;

use crate::modules::room::video::upload_video_for_room;
use crate::modules::room::websocket::handler::handle_room_socket;
use crate::{
    common::{
        error::AppError,
        response::{ApiResponse, AppResult},
    },
    state::AppState,
};

use super::{
    dto::{RoomDeleteResponse, RoomResponse},
    model::Room,
    service,
};

pub async fn get_room_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<RoomResponse> {
    let id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request("Invalid room id"))?;

    let room_collection = state.database.collection::<Room>("room");

    let response = service::get_room(room_collection, id).await?;

    Ok(ApiResponse::success(response))
}

pub async fn create_room_handler(State(state): State<AppState>) -> AppResult<RoomResponse> {
    let room_collection = state.database.collection::<Room>("room");

    let response = service::create_room(room_collection).await?;

    Ok(ApiResponse::success(response))
}

pub async fn delete_room_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<RoomDeleteResponse> {
    let id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request("Invalid room id"))?;

    let room_collection = state.database.collection::<Room>("room");

    let response = service::delete_room(room_collection, id).await?;

    Ok(ApiResponse::success(response))
}

pub async fn room_websocket_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    websocket: WebSocketUpgrade,
) -> Result<axum::response::Response, AppError> {
    let room_id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request("Invalid room id"))?;

    let room_collection = state.database.collection::<Room>("room");

    service::get_room(room_collection, room_id).await?;

    Ok(websocket.on_upgrade(move |socket| handle_room_socket(socket, state, room_id)))
}

pub async fn upload_room_video_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    multipart: Multipart,
) -> Result<Json<RoomResponse>, AppError> {
    let room_id = ObjectId::parse_str(&id)
        .map_err(|_| {
            AppError::bad_request("Invalid room id")
        })?;

    let room_collection =
        state.database.collection::<Room>("room");

    let room = upload_video_for_room(
        room_collection,
        room_id,
        multipart,
    )
    .await?;

    Ok(Json(room))
}
