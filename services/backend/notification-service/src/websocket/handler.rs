use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
    Extension,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use crate::AppState;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,  // From auth middleware
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, user_id))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>, user_id: i32) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let conn_id = state.ws_manager.add_connection(user_id, tx);

    // Send task: forward from channel to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Receive task: handle incoming messages (ping/pong)
    let ws_manager = state.ws_manager.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                axum::extract::ws::Message::Text(text) if text == "ping" => {
                    // Client keepalive
                }
                axum::extract::ws::Message::Close(_) => break,
                _ => {}
            }
        }
        ws_manager.remove_connection(user_id, conn_id);
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
