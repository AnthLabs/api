use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::{
    modules::room::controller::{
        create_room_handler, delete_room_handler, get_room_handler, room_websocket_handler,
    },
    state::AppState,
};

pub fn room_routes() -> Router<AppState> {
    let routes = Router::new()
        .route("/{id}", get(get_room_handler))
        .route("/", post(create_room_handler))
        .route("/{id}", delete(delete_room_handler))
        .route("/{id}/ws", get(room_websocket_handler));

    Router::new().nest("/room", routes)
}
