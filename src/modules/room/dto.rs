use serde::Serialize;

use super::model::Room;

#[derive(Debug, Serialize)]
pub struct RoomResponse {
    pub id: String,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

impl From<Room> for RoomResponse {
    fn from(value: Room) -> Self {
        Self {
            id: value.id.to_hex(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RoomListResponse {
    pub room_list: Vec<RoomResponse>,
    pub total: usize,
}

impl From<Vec<Room>> for RoomListResponse {
    fn from(values: Vec<Room>) -> Self {
        let total = values.len();

        Self {
            room_list: values
                .into_iter()
                .map(RoomResponse::from)
                .collect(),
            total,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RoomDeleteResponse {
    pub id: String,
}
