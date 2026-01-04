use crate::email::email_service_client::EmailServiceClient;
use crate::email::{ResendVerificationRequest, ResendVerificationResponse, VerifyCodeRequest, VerifyCodeResponse};
use crate::gateway::map_grpc_error;
use crate::AppState;
use axum::{extract::State, http::StatusCode, Json, Router, routing::post};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/verify", post(verify_email))
        .route("/resend", post(resend_verification))
}

async fn verify_email(
    State(state): State<Arc<AppState>>,
    Json(request): Json<VerifyCodeRequest>,
) -> Result<Json<VerifyCodeResponse>, StatusCode> {
    let mut client = EmailServiceClient::new(state.email_channel.clone());

    match client.verify_code(request).await {
        Ok(response) => Ok(Json(response.into_inner())),
        Err(e) => Err(map_grpc_error(e)),
    }
}

async fn resend_verification(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ResendVerificationRequest>,
) -> Result<Json<ResendVerificationResponse>, StatusCode> {
    let mut client = EmailServiceClient::new(state.email_channel.clone());

    match client.resend_verification_code(request).await {
        Ok(response) => Ok(Json(response.into_inner())),
        Err(e) => Err(map_grpc_error(e)),
    }
}
