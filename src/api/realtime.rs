use super::{dto::StreamQuery, AppState};
use axum::{
    extract::{ws::WebSocketUpgrade, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

pub(super) async fn stream(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<StreamQuery>,
) -> Response {
    let Ok(run_id) = Uuid::parse_str(&query.run_id) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    ws.on_upgrade(move |socket| handle_socket(socket, state, run_id))
}

async fn handle_socket(
    mut socket: axum::extract::ws::WebSocket,
    state: Arc<AppState>,
    run_id: Uuid,
) {
    use axum::extract::ws::Message;
    use broadcast::error::RecvError;
    if socket
        .send(Message::Text(
            json!({"type": "connected", "runId": run_id})
                .to_string()
                .into(),
        ))
        .await
        .is_err()
    {
        return;
    }
    let mut receiver = match state.stream_broadcasts.read().await.get(&run_id) {
        Some(sender) => sender.subscribe(),
        None => {
            let _ = socket
                .send(Message::Text(
                    json!({"type": "error", "error": "unknown or completed runId"})
                        .to_string()
                        .into(),
                ))
                .await;
            return;
        }
    };
    loop {
        match receiver.recv().await {
            Ok(text) => {
                if socket.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
            Err(RecvError::Lagged(_)) => continue,
            Err(RecvError::Closed) => break,
        }
    }
}
