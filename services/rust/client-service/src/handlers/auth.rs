use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{post, put},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::AppState;
use crate::models::client::{CreateClient, SocialMedia};
use crate::utils::error::ClientApiError;
use common::authorization::UserClaims;

pub mod auth {
    tonic::include_proto!("auth");
}

use auth::RegisterRequest as GrpcRegisterRequest;
use auth::ResendVerificationRequest as GrpcResendVerificationRequest;
use auth::VerifyEmailRequest as GrpcVerifyEmailRequest;
use auth::auth_service_client::AuthServiceClient;

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prenume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_info: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_media: Option<SocialMedia>,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    pub success: bool,
    pub token: String,
    pub message: String,
    pub client_id: Option<String>,
}

pub fn auth_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/verify", post(verify_email))
        .route("/resend", post(resend_verification))
}

pub fn auth_protected_router() -> Router<Arc<AppState>> {
    Router::new().route("/role/{id}", put(update_user_role))
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Email already exists"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth"
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<RegisterRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<RegisterResponse>), ClientApiError> {
    let Json(payload) = payload?;

    let mut auth_client = AuthServiceClient::connect(state.auth_service_url.clone())
        .await
        .map_err(|e| {
            ClientApiError::InternalError(format!("Failed to connect to auth service: {}", e))
        })?;

    let grpc_request = GrpcRegisterRequest {
        email: payload.email.clone(),
        password: payload.password,
        role: "client".to_string(),
    };

    let response = match auth_client.register_user(grpc_request).await {
        Ok(response) => response.into_inner(),
        Err(status) => {
            return Err(match status.code() {
                tonic::Code::InvalidArgument => {
                    ClientApiError::BadRequest(status.message().to_string())
                }
                tonic::Code::AlreadyExists => {
                    ClientApiError::Conflict(status.message().to_string())
                }
                _ => ClientApiError::InternalError(status.message().to_string()),
            });
        }
    };

    if !response.success {
        return Err(ClientApiError::AuthRegistration(response.message));
    }

    let create_client = CreateClient {
        email: payload.email.clone(),
        prenume: payload.prenume,
        nume: payload.nume,
        public_info: payload.public_info,
        social_media: payload.social_media,
    };

    create_client.validate()?;

    let created_client = state
        .client_repo
        .create_client(create_client)
        .await
        .map_err(|e| {
            ClientApiError::AuthRegistration(format!(
                "User created but failed to create client profile: {:?}",
                e
            ))
        })?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            success: true,
            token: response.token_value,
            message: "User registered successfully. Please check your email for verification code."
                .to_string(),
            client_id: Some(created_client.id.to_hex()),
        }),
    ))
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub success: bool,
    pub token: String,
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<Json<LoginResponse>, ClientApiError> {
    let Json(payload) = payload?;

    let mut auth_client = AuthServiceClient::connect(state.auth_service_url.clone())
        .await
        .map_err(|e| {
            ClientApiError::InternalError(format!("Failed to connect to auth service: {}", e))
        })?;

    let grpc_request = auth::AuthRequest {
        username: payload.email,
        password: payload.password,
    };

    let response = match auth_client.authenticate(grpc_request).await {
        Ok(response) => response.into_inner(),
        Err(status) => {
            return Err(match status.code() {
                tonic::Code::InvalidArgument => {
                    ClientApiError::BadRequest(status.message().to_string())
                }
                tonic::Code::Unauthenticated => {
                    ClientApiError::Unauthorized(status.message().to_string())
                }
                _ => ClientApiError::InternalError(status.message().to_string()),
            });
        }
    };

    if !response.success {
        return Err(ClientApiError::Unauthorized(response.message));
    }

    Ok(Json(LoginResponse {
        success: true,
        token: response.token_value,
        message: "Login successful".to_string(),
    }))
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateRoleRequest {
    pub role: String,
}

#[derive(Serialize, ToSchema)]
pub struct UpdateRoleResponse {
    pub success: bool,
    pub message: String,
}

#[utoipa::path(
    put,
    path = "/api/auth/role/{id}",
    params(
        ("id" = i32, Path, description = "User ID to update role for")
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated successfully", body = UpdateRoleResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized - admin only"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_user_role(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<i32>,
    payload: Result<Json<UpdateRoleRequest>, JsonRejection>,
) -> Result<Json<UpdateRoleResponse>, ClientApiError> {
    let Json(payload) = payload?;

    if user_claims.role != "admin" {
        return Err(ClientApiError::Forbidden(
            "Only admins can update user roles".to_string(),
        ));
    }

    let role = payload.role.to_lowercase();
    if role != "admin" && role != "client" && role != "owner-event" {
        return Err(ClientApiError::BadRequest(
            "Invalid role. Must be one of: admin, client, owner-event".to_string(),
        ));
    }

    let mut auth_client = AuthServiceClient::connect(state.auth_service_url.clone())
        .await
        .map_err(|e| {
            ClientApiError::InternalError(format!("Failed to connect to auth service: {}", e))
        })?;

    let grpc_request = auth::UpdateRoleRequest { user_id: id, role };

    let response = match auth_client.update_role(grpc_request).await {
        Ok(response) => response.into_inner(),
        Err(status) => {
            return Err(match status.code() {
                tonic::Code::InvalidArgument => {
                    ClientApiError::BadRequest(status.message().to_string())
                }
                tonic::Code::NotFound => ClientApiError::NotFound(status.message().to_string()),
                _ => ClientApiError::InternalError(status.message().to_string()),
            });
        }
    };

    if !response.success {
        return Err(ClientApiError::InternalError(response.message));
    }

    Ok(Json(UpdateRoleResponse {
        success: true,
        message: response.message,
    }))
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct VerifyEmailRequest {
    pub email: String,
    pub verification_code: String,
}

#[derive(Serialize, ToSchema)]
pub struct VerifyEmailResponse {
    pub success: bool,
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/verify",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verified successfully", body = VerifyEmailResponse),
        (status = 400, description = "Invalid or expired verification code"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth"
)]
pub async fn verify_email(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<VerifyEmailRequest>, JsonRejection>,
) -> Result<Json<VerifyEmailResponse>, ClientApiError> {
    let Json(payload) = payload?;

    let mut auth_client = AuthServiceClient::connect(state.auth_service_url.clone())
        .await
        .map_err(|e| {
            ClientApiError::InternalError(format!("Failed to connect to auth service: {}", e))
        })?;

    let grpc_request = GrpcVerifyEmailRequest {
        email: payload.email,
        verification_code: payload.verification_code,
    };

    let response = match auth_client.verify_email(grpc_request).await {
        Ok(response) => response.into_inner(),
        Err(status) => {
            return Err(match status.code() {
                tonic::Code::InvalidArgument => {
                    ClientApiError::BadRequest(status.message().to_string())
                }
                tonic::Code::NotFound => ClientApiError::NotFound(status.message().to_string()),
                _ => ClientApiError::InternalError(status.message().to_string()),
            });
        }
    };

    Ok(Json(VerifyEmailResponse {
        success: response.success,
        message: response.message,
    }))
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ResendVerificationRequest {
    pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct ResendVerificationResponse {
    pub success: bool,
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/resend",
    request_body = ResendVerificationRequest,
    responses(
        (status = 200, description = "Verification code resent", body = ResendVerificationResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth"
)]
pub async fn resend_verification(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<ResendVerificationRequest>, JsonRejection>,
) -> Result<Json<ResendVerificationResponse>, ClientApiError> {
    let Json(payload) = payload?;

    let mut auth_client = AuthServiceClient::connect(state.auth_service_url.clone())
        .await
        .map_err(|e| {
            ClientApiError::InternalError(format!("Failed to connect to auth service: {}", e))
        })?;

    let grpc_request = GrpcResendVerificationRequest {
        email: payload.email,
    };

    let response = match auth_client.resend_verification_code(grpc_request).await {
        Ok(response) => response.into_inner(),
        Err(status) => {
            return Err(match status.code() {
                tonic::Code::NotFound => ClientApiError::NotFound(status.message().to_string()),
                _ => ClientApiError::InternalError(status.message().to_string()),
            });
        }
    };

    Ok(Json(ResendVerificationResponse {
        success: response.success,
        message: response.message,
    }))
}
