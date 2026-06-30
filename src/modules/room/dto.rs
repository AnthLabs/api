use serde::Serialize;

use super::model::{VideoStatus, Room};

#[derive(Debug, Serialize)]
pub struct RoomResponse {
    pub id: String,
    pub video_url: Option<String>,
    pub video_status: VideoStatus,
    pub position_second: u32,
    pub created_at: u64,
}

impl From<Room> for RoomResponse {
    fn from(value: Room) -> Self {
        Self {
            id: value.id.to_hex(),
            video_url: value.video_url,
            video_status: value.video_status,
            position_second: value.position_second,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RoomDeleteResponse {
    pub id: String,
}
