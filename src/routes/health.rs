use axum::{Router, routing::get};

use crate::modules::health::controllers::health_check_handler;
use crate::state::AppState;

pub fn health_routes() -> Router<AppState> {
    let public_routes = Router::new().route("/", get(health_check_handler));

    Router::new().nest("/health", public_routes)
}
