use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketMessage {
    RefundStatusChanged(RefundStatusChanged),
    NewRefundRequest(NewRefundRequest),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundStatusChanged {
    pub request_id: i32,
    pub ticket_cod: String,
    pub status: String,
    pub event_name: Option<String>,
    pub message: Option<String>,
    pub user_id: i32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRefundRequest {
    pub request_id: i32,
    pub ticket_cod: String,
    pub requester_email: String,
    pub event_id: Option<i32>,
    pub packet_id: Option<i32>,
    pub reason: String,
    pub created_at: String,
    pub event_owner_id: i32,
}

pub const ROUTING_KEY_WS_BROADCAST: &str = "ws.broadcast";
pub const QUEUE_WS_BROADCAST: &str = "ws.broadcast.queue";
