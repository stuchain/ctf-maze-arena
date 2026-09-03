use super::{dto::StreamQuery, AppState};
use crate::realtime::{ServerMessage, PROTOCOL_VERSION};
use axum::{
    extract::{
        ws::{close_code, CloseFrame, Message, WebSocket},
        Query, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use std::sync::{atomic::Ordering, Arc};
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

pub(super) async fn stream(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<StreamQuery>,
) -> Response {
    let Ok(run_id) = Uuid::parse_str(&query.run_id) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    ws.on_upgrade(move |socket| handle_socket(socket, state, run_id, query.after_sequence))
}

async fn send(socket: &mut WebSocket, message: &ServerMessage) -> Result<(), ()> {
    let json = serde_json::to_string(message).map_err(|error| {
        tracing::error!(%error, "realtime message serialization failed");
    })?;
    socket
        .send(Message::Text(json.into()))
        .await
        .map_err(|_| ())
}

async fn close_for_restart(socket: &mut WebSocket, state: &AppState) {
    if !state.accepting_solves.load(Ordering::Acquire) {
        let _ = socket
            .send(Message::Close(Some(CloseFrame {
                code: close_code::RESTART,
                reason: "service restart".into(),
            })))
            .await;
    }
}

async fn handle_socket(
    mut socket: WebSocket,
    state: Arc<AppState>,
    run_id: Uuid,
    after_sequence: u64,
) {
    let Some(stream) = state.stream_broadcasts.read().await.get(&run_id).cloned() else {
        let message = ServerMessage::Failed {
            protocol_version: PROTOCOL_VERSION,
            run_id,
            sequence: 0,
            code: "stream_expired".into(),
            message: "Live history expired; load the persisted replay.".into(),
        };
        let _ = send(&mut socket, &message).await;
        return;
    };

    // Subscribe before reading retained history so publications cannot fall into a race window.
    let mut receiver = stream.subscribe();
    let batch = stream.resume(after_sequence);
    if send(&mut socket, &stream.connected(after_sequence))
        .await
        .is_err()
    {
        return;
    }
    let mut last_sent = after_sequence.min(batch.latest_sequence);
    for message in batch.messages {
        if message.sequence() <= last_sent {
            continue;
        }
        last_sent = message.sequence();
        let terminal = matches!(
            message,
            ServerMessage::Completed { .. }
                | ServerMessage::Failed { .. }
                | ServerMessage::Cancelled { .. }
        );
        if send(&mut socket, &message).await.is_err() {
            return;
        }
        if terminal {
            close_for_restart(&mut socket, &state).await;
            return;
        }
    }

    let mut heartbeat = tokio::time::interval(state.realtime_config.heartbeat_interval);
    heartbeat.tick().await;
    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                if send(&mut socket, &stream.heartbeat()).await.is_err() { return; }
            }
            received = receiver.recv() => match received {
                Ok(message) => {
                    if message.sequence() <= last_sent { continue; }
                    last_sent = message.sequence();
                    let terminal = matches!(message, ServerMessage::Completed { .. } | ServerMessage::Failed { .. } | ServerMessage::Cancelled { .. });
                    if send(&mut socket, &message).await.is_err() { return; }
                    if terminal {
                        close_for_restart(&mut socket, &state).await;
                        return;
                    }
                }
                Err(RecvError::Lagged(_)) => {
                    for message in stream.resume(last_sent).messages {
                        if message.sequence() <= last_sent { continue; }
                        last_sent = message.sequence();
                        let terminal = matches!(message, ServerMessage::Completed { .. } | ServerMessage::Failed { .. } | ServerMessage::Cancelled { .. });
                        if send(&mut socket, &message).await.is_err() { return; }
                        if terminal {
                            close_for_restart(&mut socket, &state).await;
                            return;
                        }
                    }
                }
                Err(RecvError::Closed) => return,
            }
        }
    }
}
