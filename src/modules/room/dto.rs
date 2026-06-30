use serde::Serialize;

use super::model::{Room, VideoStatus};

#[derive(Debug, Serialize, Clone)]
pub struct RoomResponse {
    pub id: String,
    pub video_url: Option<String>,
    pub video_status: VideoStatus,
    pub position_seconds: f64,
    pub created_at: u64,
    pub updated_at: Option<u64>,
}

impl From<Room> for RoomResponse {
    fn from(value: Room) -> Self {
        Self {
            id: value.id.to_hex(),
            video_url: value.video_url,
            video_status: value.video_status,
            position_seconds: value.position_seconds,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RoomDeleteResponse {
    pub id: String,
}
