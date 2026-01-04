use crate::AppState;
use crate::middleware::UserClaims;
use crate::models::refund::CreateRefundRequest;
use crate::services::event_manager;
use crate::utils::error::ClientApiError;
use axum::extract::rejection::JsonRejection;
use axum::response::IntoResponse;
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::post,
};
use common::authorization::Authorization;
use common::rabbitmq::messages::{ROUTING_KEY_REFUND_REQUESTED, RefundRequested};
use serde::Serialize;
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct RefundResponse {
    pub message: String,
}

pub fn refund_router() -> Router<Arc<AppState>> {
    Router::new().route("/clients/{id}/refunds", post(request_refund))
}

#[utoipa::path(
    post,
    path = "/api/client-manager/clients/{id}/refunds",
    params(
        ("id" = String, Path, description = "Client ID")
    ),
    request_body = CreateRefundRequest,
    responses(
        (status = 202, description = "Refund request submitted", body = RefundResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Can only request refunds on own tickets"),
        (status = 404, description = "Ticket not found in client's tickets")
    ),
    tag = "Refunds",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn request_refund(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<String>,
    payload: Result<Json<CreateRefundRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ClientApiError> {
    let client = state.client_repo.get_client(&id).await?;

    let user_email = get_user_email(&state, user_claims.user_id).await;
    Authorization::can_modify_resource(&user_claims, &client, user_email.as_deref())
        .map_err(crate::utils::error::map_authorization_error)?;

    let Json(payload) = payload?;
    payload.validate()?;

    let ticket = client
        .lista_bilete
        .iter()
        .find(|t| t.cod == payload.ticket_cod)
        .ok_or_else(|| {
            ClientApiError::NotFound(format!(
                "Ticket with code '{}' not found in this client's tickets",
                payload.ticket_cod
            ))
        })?;

    if let Some(ref status) = ticket.refund_status {
        if status == "PENDING" {
            return Err(ClientApiError::Conflict(
                "A refund request is already pending for this ticket".to_string(),
            ));
        }
        if status == "APPROVED" {
            return Err(ClientApiError::Conflict(
                "This ticket has already been refunded".to_string(),
            ));
        }
        if status == "REJECTED" {
            return Err(ClientApiError::Conflict(
                "A refund request for this ticket was already rejected. You cannot request another refund.".to_string(),
            ));
        }
    }

    let ticket_details = event_manager::get_ticket_details(
        &state.event_manager_client,
        &payload.ticket_cod,
        &state.service_token,
    )
    .await
    .map_err(|e| {
        error!("Failed to get ticket details: {:?}", e);
        ClientApiError::InternalError("Failed to get ticket details".to_string())
    })?;

    let event_id = ticket_details.ticket.evenimentid;
    let packet_id = ticket_details.ticket.pachetid;

    let event_owner_id = if let Some(ref event) = ticket_details.event {
        event.id_owner
    } else if let Some(ref packet) = ticket_details.packet {
        packet.id_owner
    } else {
        return Err(ClientApiError::BadRequest(
            "Ticket is not associated with any event or package".to_string(),
        ));
    };

    let message = RefundRequested {
        request_id: 0,
        ticket_cod: payload.ticket_cod.clone(),
        requester_id: user_claims.user_id,
        requester_email: client.email.clone(),
        event_id,
        packet_id,
        event_owner_id,
        reason: payload.reason.clone(),
    };

    let json = serde_json::to_vec(&message).map_err(|e| {
        error!("Failed to serialize refund request: {:?}", e);
        ClientApiError::InternalError("Failed to serialize refund request".to_string())
    })?;

    state
        .rabbitmq
        .publish(ROUTING_KEY_REFUND_REQUESTED, &json)
        .await
        .map_err(|e| {
            error!("Failed to publish refund request: {:?}", e);
            ClientApiError::InternalError("Failed to submit refund request".to_string())
        })?;

    if let Err(e) = state
        .client_repo
        .update_ticket_refund_status(&id, &payload.ticket_cod, Some("PENDING"))
        .await
    {
        error!("Failed to update ticket refund status: {:?}", e);
    }

    info!(
        "Refund request submitted for ticket {} by client {}",
        payload.ticket_cod, id
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(RefundResponse {
            message:
                "Refund request submitted successfully. The event owner will review your request."
                    .to_string(),
        }),
    ))
}

async fn get_user_email(state: &AppState, user_id: i32) -> Option<String> {
    use crate::handlers::client::auth::GetUserEmailRequest;
    use crate::handlers::client::auth::auth_service_client::AuthServiceClient;

    let mut auth_client = match AuthServiceClient::connect(state.auth_service_url.clone()).await {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to connect to auth service: {}", e);
            return None;
        }
    };

    let request = GetUserEmailRequest { user_id };

    let response = match auth_client.get_user_email(request).await {
        Ok(resp) => resp.into_inner(),
        Err(e) => {
            tracing::error!("Failed to get user email from auth service: {}", e);
            return None;
        }
    };

    if response.success {
        Some(response.email)
    } else {
        None
    }
}
