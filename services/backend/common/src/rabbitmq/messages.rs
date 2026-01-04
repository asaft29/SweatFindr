use serde::{Deserialize, Serialize};

pub const ROUTING_KEY_REFUND_REQUESTED: &str = "refund.requested";
pub const ROUTING_KEY_REFUND_RESOLVED: &str = "refund.resolved";

pub const QUEUE_REFUND_REQUESTED: &str = "refund.requested.queue";
pub const QUEUE_REFUND_RESOLVED_EMAIL: &str = "refund.resolved.email.queue";
pub const QUEUE_REFUND_RESOLVED_CLIENT: &str = "refund.resolved.client.queue";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequested {
    pub request_id: i32,
    pub ticket_cod: String,
    pub requester_id: i32,
    pub requester_email: String,
    pub event_id: Option<i32>,
    pub packet_id: Option<i32>,
    pub event_owner_id: i32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResolved {
    pub request_id: i32,
    pub ticket_cod: String,
    pub requester_email: String,
    pub status: RefundStatus,
    pub event_name: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RefundStatus {
    Approved,
    Rejected,
}
