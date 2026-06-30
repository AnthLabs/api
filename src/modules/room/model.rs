use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum VideoStatus {
    Paused,
    Playing,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Room {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub video_url: Option<String>,
    pub video_status: VideoStatus,
    pub position_seconds: f64,
    pub created_at: u64,
    pub updated_at: Option<u64>,
}
