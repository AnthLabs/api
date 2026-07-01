use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use mongodb::{Collection, bson::oid::ObjectId};

use crate::{
    modules::room::{
        model::Room,
        service::{self, PlaybackCommand},
    },
    state::AppState,
};

use super::message::{ClientMessage, ServerMessage};

pub async fn handle_room_socket(socket: WebSocket, state: AppState, room_id: ObjectId) {
    let room_collection = state.database.collection::<Room>("room");

    let initial_room = match service::get_room(room_collection.clone(), room_id).await {
        Ok(room) => room,

        Err(error) => {
            send_initial_error(socket, "room_not_found", error.message).await;

            return;
        }
    };

    let mut receiver = state.room_hub.subscribe(room_id).await;

    let (mut socket_sender, mut socket_receiver) = socket.split();

    let initial_message = ServerMessage::RoomUpdated { room: initial_room };

    if send_server_message(&mut socket_sender, &initial_message)
        .await
        .is_err()
    {
        return;
    }

    let mut socket_send_task = tokio::spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(message) => {
                    if send_server_message(&mut socket_sender, &message)
                        .await
                        .is_err()
                    {
                        break;
                    }
                }

                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }

                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    let receive_state = state.clone();

    let mut socket_receive_task = tokio::spawn(async move {
        while let Some(result) = socket_receiver.next().await {
            let message = match result {
                Ok(message) => message,
                Err(_) => break,
            };

            match message {
                Message::Text(text) => {
                    process_text_message(
                        &receive_state,
                        room_collection.clone(),
                        room_id,
                        text.as_str(),
                    )
                    .await;
                }

                Message::Close(_) => {
                    break;
                }

                Message::Ping(_) => {}

                Message::Pong(_) => {}

                Message::Binary(_) => {}
            }
        }
    });

    tokio::select! {
        _ = &mut socket_send_task => {
            socket_receive_task.abort();
        }

        _ = &mut socket_receive_task => {
            socket_send_task.abort();
        }
    }
}

async fn process_text_message(
    state: &AppState,
    room_collection: Collection<Room>,
    room_id: ObjectId,
    text: &str,
) {
    let client_message = match serde_json::from_str::<ClientMessage>(text) {
        Ok(message) => message,

        Err(_) => {
            state
                .room_hub
                .broadcast(
                    room_id,
                    ServerMessage::Error {
                        code: "invalid_message".to_string(),
                        message: "Invalid WebSocket message".to_string(),
                    },
                )
                .await;

            return;
        }
    };

    match client_message {
        ClientMessage::ChangeVideo { video_url } => {
            apply_and_broadcast(
                state,
                room_collection,
                room_id,
                PlaybackCommand::ChangeVideo { video_url },
            )
            .await;
        }

        ClientMessage::Play { position_seconds } => {
            apply_and_broadcast(
                state,
                room_collection,
                room_id,
                PlaybackCommand::Play { position_seconds },
            )
            .await;
        }

        ClientMessage::Pause { position_seconds } => {
            apply_and_broadcast(
                state,
                room_collection,
                room_id,
                PlaybackCommand::Pause { position_seconds },
            )
            .await;
        }

        ClientMessage::Seek { position_seconds } => {
            apply_and_broadcast(
                state,
                room_collection,
                room_id,
                PlaybackCommand::Seek { position_seconds },
            )
            .await;
        }
    }
}

async fn apply_and_broadcast(
    state: &AppState,
    room_collection: Collection<Room>,
    room_id: ObjectId,
    command: PlaybackCommand,
) {
    let command_for_log = command.clone();

    match service::apply_playback_command(room_collection, room_id, command).await {
        Ok(room) => {
            if let Err(error) = log_playback_command(state, room_id, &command_for_log).await {
                eprintln!("Failed to write room action log: {}", error.message,);

                state
                    .room_hub
                    .broadcast(
                        room_id,
                        ServerMessage::Error {
                            code: "room_log_failed".to_string(),
                            message: error.message,
                        },
                    )
                    .await;

                return;
            }

            state
                .room_hub
                .broadcast(room_id, ServerMessage::RoomUpdated { room })
                .await;
        }

        Err(error) => {
            state
                .room_hub
                .broadcast(
                    room_id,
                    ServerMessage::Error {
                        code: "room_update_failed".to_string(),
                        message: error.message,
                    },
                )
                .await;
        }
    }
}

async fn log_playback_command(
    state: &AppState,
    room_id: ObjectId,
    command: &PlaybackCommand,
) -> Result<(), crate::common::error::AppError> {
    match command {
        PlaybackCommand::ChangeVideo { video_url } => {
            state
                .room_logger
                .log_video_changed(room_id, video_url)
                .await
        }

        PlaybackCommand::Play { position_seconds } => {
            state.room_logger.log_play(room_id, *position_seconds).await
        }

        PlaybackCommand::Pause { position_seconds } => {
            state
                .room_logger
                .log_pause(room_id, *position_seconds)
                .await
        }

        PlaybackCommand::Seek { position_seconds } => {
            state.room_logger.log_seek(room_id, *position_seconds).await
        }
    }
}

async fn send_server_message<S>(sender: &mut S, message: &ServerMessage) -> Result<(), ()>
where
    S: futures_util::Sink<Message, Error = axum::Error> + Unpin,
{
    let serialized = serde_json::to_string(message).map_err(|_| ())?;

    sender
        .send(Message::Text(serialized.into()))
        .await
        .map_err(|_| ())
}

async fn send_initial_error(mut socket: WebSocket, code: &str, message: String) {
    let response = ServerMessage::Error {
        code: code.to_string(),
        message,
    };

    let Ok(serialized) = serde_json::to_string(&response) else {
        return;
    };

    let _ = socket.send(Message::Text(serialized.into())).await;

    let _ = socket.close().await;
}
