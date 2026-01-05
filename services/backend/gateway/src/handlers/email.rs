use crate::AppState;
use crate::auth::ResetPasswordRequest;
use crate::auth::auth_service_client::AuthServiceClient;
use crate::email::email_service_client::EmailServiceClient;
use crate::email::{
    ResendVerificationRequest, ResendVerificationResponse, SendPasswordResetRequest,
    SendPasswordResetResponse, VerifyCodeRequest, VerifyCodeResponse,
    VerifyPasswordResetCodeRequest, VerifyPasswordResetCodeResponse,
};
use crate::gateway::map_grpc_error;
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/verify", post(verify_email))
        .route("/resend", post(resend_verification))
        .route("/forgot-password", post(forgot_password))
        .route("/verify-reset-code", post(verify_reset_code))
        .route("/reset-password", post(reset_password))
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

async fn forgot_password(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendPasswordResetRequest>,
) -> Result<Json<SendPasswordResetResponse>, StatusCode> {
    let mut client = EmailServiceClient::new(state.email_channel.clone());

    match client.send_password_reset_email(request).await {
        Ok(response) => Ok(Json(response.into_inner())),
        Err(e) => Err(map_grpc_error(e)),
    }
}

async fn verify_reset_code(
    State(state): State<Arc<AppState>>,
    Json(request): Json<VerifyPasswordResetCodeRequest>,
) -> Result<Json<VerifyPasswordResetCodeResponse>, StatusCode> {
    let mut client = EmailServiceClient::new(state.email_channel.clone());

    match client.verify_password_reset_code(request).await {
        Ok(response) => Ok(Json(response.into_inner())),
        Err(e) => Err(map_grpc_error(e)),
    }
}

#[derive(Deserialize)]
struct ResetPasswordPayload {
    email: String,
    new_password: String,
    reset_token: String,
}

#[derive(Serialize)]
struct ResetPasswordResponse {
    success: bool,
    message: String,
}

async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResetPasswordPayload>,
) -> Result<Json<ResetPasswordResponse>, StatusCode> {
    let mut client = AuthServiceClient::new(state.auth_channel.clone());

    let request = ResetPasswordRequest {
        email: payload.email,
        new_password: payload.new_password,
        reset_token: payload.reset_token,
    };

    match client.reset_password(request).await {
        Ok(response) => {
            let inner = response.into_inner();
            Ok(Json(ResetPasswordResponse {
                success: inner.success,
                message: inner.message,
            }))
        }
        Err(e) => Err(map_grpc_error(e)),
    }
}
