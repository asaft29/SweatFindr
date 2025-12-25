use crate::models::email::*;
use crate::AppState;
use crate::email::email_service_client::EmailServiceClient;
use axum::{extract::State, http::StatusCode, Json, Router, routing::post};
use std::sync::Arc;
use tracing::error;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/verify", post(verify_email))
        .route("/resend", post(resend_verification))
}

async fn verify_email(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<Json<VerifyEmailResponse>, StatusCode> {
    let mut client = EmailServiceClient::connect(state.email_service_url.clone())
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to connect to email service");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    let grpc_request = crate::email::VerifyCodeRequest {
        user_id: payload.user_id,
        verification_code: payload.verification_code,
    };

    match client.verify_code(grpc_request).await {
        Ok(response) => {
            let res = response.into_inner();
            Ok(Json(VerifyEmailResponse {
                success: res.success,
                message: res.message,
            }))
        }
        Err(e) => {
            error!(error = %e, "gRPC error during email verification");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn resend_verification(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResendVerificationRequest>,
) -> Result<Json<ResendVerificationResponse>, StatusCode> {
    let mut client = EmailServiceClient::connect(state.email_service_url.clone())
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to connect to email service");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    let grpc_request = crate::email::ResendVerificationRequest {
        user_id: payload.user_id,
        email: payload.email,
    };

    match client.resend_verification_code(grpc_request).await {
        Ok(response) => {
            let res = response.into_inner();
            Ok(Json(ResendVerificationResponse {
                success: res.success,
                message: res.message,
            }))
        }
        Err(e) => {
            error!(error = %e, "gRPC error during resend verification");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
