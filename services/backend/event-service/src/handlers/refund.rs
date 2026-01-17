use crate::AppState;
use crate::middleware::UserClaims;
use crate::models::refund::{RefundRequest, RejectRefundRequest};
use crate::utils::error::{ApiError, RefundRepoError};
use axum::extract::rejection::JsonRejection;
use axum::response::IntoResponse;
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use common::rabbitmq::messages::{ROUTING_KEY_REFUND_RESOLVED, RefundResolved, RefundStatus};
use common::websocket::messages::{ROUTING_KEY_WS_BROADCAST, RefundStatusChanged, WebSocketMessage};
use std::sync::Arc;
use tracing::{error, info, warn};
use validator::Validate;

pub fn refund_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/refunds", get(list_pending_refunds))
        .route("/refunds/history", get(list_client_refund_history))
        .route("/refunds/{id}", get(get_refund))
        .route("/refunds/{id}/approve", post(approve_refund))
        .route("/refunds/{id}/reject", post(reject_refund))
}

fn map_refund_error(e: crate::repositories::refund_repo::RefundRepoError) -> ApiError {
    use crate::repositories::refund_repo::RefundRepoError as RepoError;
    match e {
        RepoError::NotFound => ApiError::Refund(RefundRepoError::NotFound),
        RepoError::InternalError(err) => {
            error!("Database error: {:?}", err);
            ApiError::Refund(RefundRepoError::InternalError(err))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/event-manager/refunds",
    responses(
        (status = 200, description = "List of pending refund requests", body = Vec<RefundRequest>),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Only event owners can view refunds")
    ),
    tag = "Refunds",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_pending_refunds(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
) -> Result<impl IntoResponse, ApiError> {
    if !user_claims.is_owner_event() && !user_claims.is_admin() {
        return Err(ApiError::Forbidden(
            "Only event owners can view refund requests".to_string(),
        ));
    }

    let refunds = state
        .refund_repo
        .list_pending_refunds_for_owner(user_claims.user_id)
        .await
        .map_err(map_refund_error)?;

    Ok(Json(serde_json::json!({ "data": refunds })))
}

#[derive(Debug, serde::Deserialize)]
pub struct RefundHistoryQuery {
    pub email: String,
}

#[utoipa::path(
    get,
    path = "/api/event-manager/refunds/history",
    params(
        ("email" = String, Query, description = "Requester email address")
    ),
    responses(
        (status = 200, description = "List of refund requests for the email", body = Vec<RefundRequest>),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Can only view own refund history")
    ),
    tag = "Refunds",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_client_refund_history(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    axum::extract::Query(query): axum::extract::Query<RefundHistoryQuery>,
) -> Result<impl IntoResponse, ApiError> {
    if !user_claims.is_client() && !user_claims.is_admin() {
        return Err(ApiError::Forbidden(
            "Only clients can view their refund history".to_string(),
        ));
    }

    let refunds = state
        .refund_repo
        .list_refunds_by_requester_email(&query.email)
        .await
        .map_err(map_refund_error)?;

    Ok(Json(serde_json::json!({ "data": refunds })))
}

#[utoipa::path(
    get,
    path = "/api/event-manager/refunds/{id}",
    params(
        ("id" = i32, Path, description = "Refund request ID")
    ),
    responses(
        (status = 200, description = "Refund request found", body = RefundRequest),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Can only view own refunds"),
        (status = 404, description = "Refund request not found")
    ),
    tag = "Refunds",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_refund(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let refund = state
        .refund_repo
        .get_refund_request(id)
        .await
        .map_err(map_refund_error)?;

    if refund.event_owner_id != user_claims.user_id && !user_claims.is_admin() {
        return Err(ApiError::Forbidden(
            "You can only view refunds for your own events".to_string(),
        ));
    }

    Ok(Json(refund))
}

#[utoipa::path(
    post,
    path = "/api/event-manager/refunds/{id}/approve",
    params(
        ("id" = i32, Path, description = "Refund request ID")
    ),
    responses(
        (status = 200, description = "Refund request approved", body = RefundRequest),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Can only approve own refunds"),
        (status = 404, description = "Refund request not found or already resolved")
    ),
    tag = "Refunds",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn approve_refund(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if !user_claims.is_owner_event() && !user_claims.is_admin() {
        return Err(ApiError::Forbidden(
            "Only event owners can approve refunds".to_string(),
        ));
    }

    let refund = state
        .refund_repo
        .approve_refund(id, user_claims.user_id)
        .await
        .map_err(map_refund_error)?;

    match state.ticket_repo.delete_ticket(&refund.ticket_cod).await {
        Ok(_) => {
            info!(
                "Deleted ticket {} and restored seat count (refund approved)",
                refund.ticket_cod
            );
        }
        Err(e) => {
            warn!(
                "Failed to delete ticket {} from BILETE table: {:?}. Seat count may not be restored.",
                refund.ticket_cod, e
            );
        }
    }

    let event_name = state.refund_repo.get_event_name_for_refund(&refund).await;

    let message = RefundResolved {
        request_id: refund.id,
        ticket_cod: refund.ticket_cod.clone(),
        requester_email: refund.requester_email.clone(),
        status: RefundStatus::Approved,
        event_name,
        message: None,
    };

    if let Ok(json) = serde_json::to_vec(&message) {
        if let Err(e) = state
            .rabbitmq
            .publish(ROUTING_KEY_REFUND_RESOLVED, &json)
            .await
        {
            error!("Failed to publish refund.resolved message: {:?}", e);
        } else {
            info!(
                "Published refund.resolved (approved) for request {}",
                refund.id
            );
        }
    }


    let ws_message = WebSocketMessage::RefundStatusChanged(RefundStatusChanged {
        request_id: refund.id,
        ticket_cod: refund.ticket_cod.clone(),
        status: "APPROVED".to_string(),
        event_name: state.refund_repo.get_event_name_for_refund(&refund).await,
        message: Some("Your refund has been approved".to_string()),
        user_id: refund.requester_id,
    });

    if let Ok(json) = serde_json::to_vec(&ws_message) {
        if let Err(e) = state
            .rabbitmq
            .publish(ROUTING_KEY_WS_BROADCAST, &json)
            .await
        {
            error!("Failed to publish WebSocket message: {:?}", e);
        } else {
            info!("Published WebSocket notification for refund approval {}", refund.id);
        }
    }

    Ok(Json(refund))
}

#[utoipa::path(
    post,
    path = "/api/event-manager/refunds/{id}/reject",
    params(
        ("id" = i32, Path, description = "Refund request ID")
    ),
    request_body = RejectRefundRequest,
    responses(
        (status = 200, description = "Refund request rejected", body = RefundRequest),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Can only reject own refunds"),
        (status = 404, description = "Refund request not found or already resolved"),
        (status = 422, description = "Validation error")
    ),
    tag = "Refunds",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn reject_refund(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<i32>,
    payload: Result<Json<RejectRefundRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    if !user_claims.is_owner_event() && !user_claims.is_admin() {
        return Err(ApiError::Forbidden(
            "Only event owners can reject refunds".to_string(),
        ));
    }

    let Json(payload) = payload?;
    payload.validate()?;

    let rejection_message = payload.message.clone();

    let refund = state
        .refund_repo
        .reject_refund(id, user_claims.user_id, &payload.message)
        .await
        .map_err(map_refund_error)?;

    let event_name = state.refund_repo.get_event_name_for_refund(&refund).await;

    let message = RefundResolved {
        request_id: refund.id,
        ticket_cod: refund.ticket_cod.clone(),
        requester_email: refund.requester_email.clone(),
        status: RefundStatus::Rejected,
        event_name: event_name.clone(),
        message: Some(rejection_message.clone()),
    };

    if let Ok(json) = serde_json::to_vec(&message) {
        if let Err(e) = state
            .rabbitmq
            .publish(ROUTING_KEY_REFUND_RESOLVED, &json)
            .await
        {
            error!("Failed to publish refund.resolved message: {:?}", e);
        } else {
            info!(
                "Published refund.resolved (rejected) for request {}",
                refund.id
            );
        }
    }


    let ws_message = WebSocketMessage::RefundStatusChanged(RefundStatusChanged {
        request_id: refund.id,
        ticket_cod: refund.ticket_cod.clone(),
        status: "REJECTED".to_string(),
        event_name,
        message: Some(rejection_message),
        user_id: refund.requester_id,
    });

    if let Ok(json) = serde_json::to_vec(&ws_message) {
        if let Err(e) = state
            .rabbitmq
            .publish(ROUTING_KEY_WS_BROADCAST, &json)
            .await
        {
            error!("Failed to publish WebSocket message: {:?}", e);
        } else {
            info!("Published WebSocket notification for refund rejection {}", refund.id);
        }
    }

    Ok(Json(refund))
}
