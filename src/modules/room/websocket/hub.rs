use std::{collections::HashMap, sync::Arc};

use mongodb::bson::oid::ObjectId;
use tokio::sync::{RwLock, broadcast};

use super::message::ServerMessage;

const ROOM_CHANNEL_CAPACITY: usize = 128;

#[derive(Debug, Clone)]
pub struct RoomHub {
    rooms: Arc<RwLock<HashMap<ObjectId, broadcast::Sender<ServerMessage>>>>,
}

impl RoomHub {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn sender(&self, room_id: ObjectId) -> broadcast::Sender<ServerMessage> {
        {
            let rooms = self.rooms.read().await;

            if let Some(sender) = rooms.get(&room_id) {
                return sender.clone();
            }
        }

        let mut rooms = self.rooms.write().await;

        rooms
            .entry(room_id)
            .or_insert_with(|| {
                let (sender, _) = broadcast::channel(ROOM_CHANNEL_CAPACITY);

                sender
            })
            .clone()
    }

    pub async fn subscribe(&self, room_id: ObjectId) -> broadcast::Receiver<ServerMessage> {
        self.sender(room_id).await.subscribe()
    }

    pub async fn broadcast(&self, room_id: ObjectId, message: ServerMessage) {
        let sender = self.sender(room_id).await;

        // Une absence de récepteur n'est pas une erreur métier.
        let _ = sender.send(message);
    }

    pub async fn remove_room(&self, room_id: &ObjectId) {
        let mut rooms = self.rooms.write().await;
        rooms.remove(room_id);
    }
}

impl Default for RoomHub {
    fn default() -> Self {
        Self::new()
    }
}
