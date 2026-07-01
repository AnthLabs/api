use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use mongodb::bson::oid::ObjectId;
use serde::Serialize;
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
    sync::{Mutex, RwLock},
};

use crate::common::{
    error::AppError,
    media::logs_directory,
};

#[derive(Debug, Clone)]
pub struct RoomLogger {
    room_locks: Arc<RwLock<HashMap<ObjectId, Arc<Mutex<()>>>>>,
}

#[derive(Debug, Serialize)]
pub struct RoomLogEntry<'a> {
    pub timestamp: u64,
    pub event: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_url: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_seconds: Option<f64>,
}

impl RoomLogger {
    pub fn new() -> Self {
        Self {
            room_locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn log_room_created(
        &self,
        room_id: ObjectId,
    ) -> Result<(), AppError> {
        self.write_entry(
            room_id,
            "room_created",
            None,
            None,
        )
        .await
    }

    pub async fn log_video_uploaded(
        &self,
        room_id: ObjectId,
    ) -> Result<(), AppError> {
        self.write_entry(
            room_id,
            "video_uploaded",
            None,
            None,
        )
        .await
    }

    pub async fn log_video_changed(
        &self,
        room_id: ObjectId,
        video_url: &str,
    ) -> Result<(), AppError> {
        self.write_entry(
            room_id,
            "video_changed",
            Some(video_url),
            None,
        )
        .await
    }

    pub async fn log_play(
        &self,
        room_id: ObjectId,
        position_seconds: f64,
    ) -> Result<(), AppError> {
        self.write_entry(
            room_id,
            "play",
            None,
            Some(position_seconds),
        )
        .await
    }

    pub async fn log_pause(
        &self,
        room_id: ObjectId,
        position_seconds: f64,
    ) -> Result<(), AppError> {
        self.write_entry(
            room_id,
            "pause",
            None,
            Some(position_seconds),
        )
        .await
    }

    pub async fn log_seek(
        &self,
        room_id: ObjectId,
        position_seconds: f64,
    ) -> Result<(), AppError> {
        self.write_entry(
            room_id,
            "seek",
            None,
            Some(position_seconds),
        )
        .await
    }

    async fn write_entry(
        &self,
        room_id: ObjectId,
        event: &str,
        video_url: Option<&str>,
        position_seconds: Option<f64>,
    ) -> Result<(), AppError> {
        let room_lock = self.room_lock(room_id).await;
        let _guard = room_lock.lock().await;

        let logs_directory = logs_directory();

        fs::create_dir_all(&logs_directory)
            .await
            .map_err(|error| {
                eprintln!(
                    "Failed to create room log directory {}: {error}",
                    logs_directory.display(),
                );

                AppError::internal(
                    "Failed to create room log directory",
                )
            })?;

        let log_path = logs_directory.join(
            format!("{}.log", room_id.to_hex()),
        );

        let entry = RoomLogEntry {
            timestamp: unix_timestamp()?,
            event,
            video_url,
            position_seconds,
        };

        let mut serialized =
            serde_json::to_vec(&entry).map_err(|error| {
                eprintln!(
                    "Failed to serialize room log entry: {error}"
                );

                AppError::internal(
                    "Failed to serialize room log entry",
                )
            })?;

        serialized.push(b'\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await
            .map_err(|error| {
                eprintln!(
                    "Failed to open room log file {}: {error}",
                    log_path.display(),
                );

                AppError::internal(
                    "Failed to open room log file",
                )
            })?;

        file.write_all(&serialized)
            .await
            .map_err(|error| {
                eprintln!(
                    "Failed to write room log file {}: {error}",
                    log_path.display(),
                );

                AppError::internal(
                    "Failed to write room log entry",
                )
            })?;

        file.flush()
            .await
            .map_err(|error| {
                eprintln!(
                    "Failed to flush room log file {}: {error}",
                    log_path.display(),
                );

                AppError::internal(
                    "Failed to flush room log entry",
                )
            })?;

        Ok(())
    }

    async fn room_lock(
        &self,
        room_id: ObjectId,
    ) -> Arc<Mutex<()>> {
        {
            let room_locks = self.room_locks.read().await;

            if let Some(room_lock) = room_locks.get(&room_id) {
                return room_lock.clone();
            }
        }

        let mut room_locks = self.room_locks.write().await;

        room_locks
            .entry(room_id)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
}

impl Default for RoomLogger {
    fn default() -> Self {
        Self::new()
    }
}

fn unix_timestamp() -> Result<u64, AppError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| {
            AppError::internal(
                "System time is before UNIX_EPOCH",
            )
        })
}
