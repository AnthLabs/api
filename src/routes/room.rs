use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, get, post},
};

use crate::{
    modules::room::controller::{
        create_room_handler, delete_room_handler, get_room_handler, room_websocket_handler,
        upload_room_video_handler,
    },
    state::AppState,
};

pub fn room_routes() -> Router<AppState> {
    let routes = Router::new()
        .route("/{id}", get(get_room_handler))
        .route("/", post(create_room_handler))
        .route("/{id}", delete(delete_room_handler))
        .route("/{id}/ws", get(room_websocket_handler))
        .route("/{id}/upload_video", post(upload_room_video_handler))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024 * 1024));
    Router::new().nest("/room", routes)
}
