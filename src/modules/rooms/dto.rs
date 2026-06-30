use serde::Serialize;

use super::model::Rooms;

#[derive(Debug, Serialize)]
pub struct RoomsResponse {
    pub id: String,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

impl From<Rooms> for RoomsResponse {
    fn from(value: Rooms) -> Self {
        Self {
            id: value.id.to_hex(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RoomsListResponse {
    pub rooms_list: Vec<RoomsResponse>,
    pub total: usize,
}

impl From<Vec<Rooms>> for RoomsListResponse {
    fn from(values: Vec<Rooms>) -> Self {
        let total = values.len();

        Self {
            rooms_list: values
                .into_iter()
                .map(RoomsResponse::from)
                .collect(),
            total,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RoomsDeleteResponse {
    pub id: String,
}
