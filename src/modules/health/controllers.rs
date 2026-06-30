use axum::extract::State;

use crate::{
    common::response::{ApiResponse, AppResult},
    modules::health::{dto::HealthCheckResponse, services},
    state::AppState,
};

pub async fn health_check_handler(State(state): State<AppState>) -> AppResult<HealthCheckResponse> {
    let response = services::health_check(state.started_at)?;
    Ok(ApiResponse::success(response))
}
