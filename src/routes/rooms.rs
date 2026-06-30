use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use crate::{
    modules::rooms::controller::{
        create_rooms_handler,
        delete_rooms_handler,
        get_rooms_handler,
        get_all_rooms_handler,
        update_rooms_handler,
    },
    state::AppState,
};

pub fn rooms_routes() -> Router<AppState> {
    let routes = Router::new()
        .route("/", get(get_all_rooms_handler))
        .route("/{id}", get(get_rooms_handler))
        .route("/", post(create_rooms_handler))
        .route("/{id}", patch(update_rooms_handler))
        .route("/{id}", delete(delete_rooms_handler));

    Router::new().nest("/rooms", routes)
}
