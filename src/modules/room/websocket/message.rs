use serde::{Deserialize, Serialize};

use crate::modules::room::dto::RoomResponse;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    ChangeVideo { video_url: String },
    Play { position_seconds: f64 },
    Pause { position_seconds: f64 },
    Seek { position_seconds: f64 },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    RoomUpdated { room: RoomResponse },
    Error { code: String, message: String },
}
