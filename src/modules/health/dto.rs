use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub uptime_seconds: u64,
    pub version: String,
}
