use crate::{common::error::AppError, modules::health::dto::HealthCheckResponse};

pub fn health_check(started_at: std::time::Instant) -> Result<HealthCheckResponse, AppError> {
    Ok(HealthCheckResponse {
        status: "healthy".into(),
        uptime_seconds: started_at.elapsed().as_secs(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}
