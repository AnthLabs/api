use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum VideoStatus{
    Paused,
    Playing
}

impl From<String> for VideoStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "paused" => VideoStatus::Paused,
            "playing" => VideoStatus::Playing,
            _ => VideoStatus::Paused,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Room {
 #[serde(rename = "_id")]
    pub id: ObjectId,
    pub video_url: Option<String>,
    pub video_status: VideoStatus,
    pub position_second: u32,
    pub created_at: u64,
}
