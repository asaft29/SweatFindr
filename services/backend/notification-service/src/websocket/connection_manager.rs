use axum::extract::ws::Message;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

pub type ConnectionId = Uuid;
pub type UserId = i32;

#[derive(Clone)]
pub struct ConnectionManager {
    connections:
        Arc<DashMap<UserId, Vec<(ConnectionId, tokio::sync::mpsc::UnboundedSender<Message>)>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
        }
    }

    pub fn add_connection(
        &self,
        user_id: UserId,
        sender: tokio::sync::mpsc::UnboundedSender<Message>,
    ) -> ConnectionId {
        let conn_id = Uuid::now_v7();
        self.connections
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push((conn_id, sender));
        info!("User {} connected (connection {})", user_id, conn_id);
        conn_id
    }

    pub fn remove_connection(&self, user_id: UserId, conn_id: ConnectionId) {
        if let Some(mut entry) = self.connections.get_mut(&user_id) {
            entry.retain(|(id, _)| *id != conn_id);
            if entry.is_empty() {
                drop(entry);
                self.connections.remove(&user_id);
            }
        }
        info!("User {} disconnected (connection {})", user_id, conn_id);
    }

    pub async fn broadcast_to_user(&self, user_id: UserId, message: &str) {
        if let Some(entry) = self.connections.get(&user_id) {
            let msg = Message::Text(message.to_string().into());
            for (conn_id, sender) in entry.value() {
                if sender.send(msg.clone()).is_err() {
                    warn!("Failed to send to user {} connection {}", user_id, conn_id);
                }
            }
        }
    }

    pub fn connected_users_count(&self) -> usize {
        self.connections.len()
    }
}
