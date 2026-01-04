use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RefundRequest {
    pub id: i32,
    pub ticket_cod: String,
    pub requester_id: i32,
    pub requester_email: String,
    pub event_id: Option<i32>,
    pub packet_id: Option<i32>,
    pub event_owner_id: i32,
    pub status: String,
    pub reason: Option<String>,
    pub rejection_message: Option<String>,
    pub created_at: Option<String>,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateRefundRequest {
    pub ticket_cod: String,
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Reason must be between 1 and 1000 characters"
    ))]
    pub reason: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RejectRefundRequest {
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Message must be between 1 and 1000 characters"
    ))]
    pub message: String,
}
