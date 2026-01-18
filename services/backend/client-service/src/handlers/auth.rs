use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{post, put},
};
use validator::Validate;

use crate::AppState;
use crate::models::auth::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, ResendVerificationRequest,
    ResendVerificationResponse, UpdateRoleRequest, UpdateRoleResponse, VerifyEmailRequest,
    VerifyEmailResponse,
};
use crate::models::client::CreateClient;
use crate::utils::auth_links::{
    Response, build_login_response, build_register_response, build_resend_verification_response,
    build_update_role_response, build_verify_email_response,
};
use crate::utils::error::ClientApiError;
use common::authorization::UserClaims;

pub mod auth {
    tonic::include_proto!("auth");
}

pub mod email {
    tonic::include_proto!("email");
}

use auth::RegisterRequest as GrpcRegisterRequest;
use auth::auth_service_client::AuthServiceClient;
use email::email_service_client::EmailServiceClient;
use email::{
    ResendVerificationRequest as EmailResendRequest, SendVerificationRequest, VerifyCodeRequest,
};

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
        (status = 201, description = "User registered successfully", body = Response<RegisterResponse>),
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
) -> Result<(StatusCode, Json<Response<RegisterResponse>>), ClientApiError> {
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

    let mut email_client = EmailServiceClient::connect(state.email_service_url.clone())
        .await
        .map_err(|e| {
            ClientApiError::InternalError(format!("Failed to connect to email service: {}", e))
        })?;

    let email_request = SendVerificationRequest {
        user_id: response.user_id,
        email: payload.email.clone(),
    };

    tokio::spawn(async move {
        if let Err(e) = email_client.send_verification_email(email_request).await {
            eprintln!("Failed to send verification email: {}", e);
        }
    });

    let register_response = RegisterResponse {
        success: true,
        token: response.token_value,
        message: "User registered successfully. Please check your email for verification code."
            .to_string(),
        client_id: Some(created_client.id.to_hex()),
    };

    Ok((
        StatusCode::CREATED,
        Json(build_register_response(register_response, &state.base_url)),
    ))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = Response<LoginResponse>),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<Json<Response<LoginResponse>>, ClientApiError> {
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

    let login_response = LoginResponse {
        success: true,
        token: response.token_value,
        message: "Login successful".to_string(),
    };

    Ok(Json(build_login_response(login_response, &state.base_url)))
}

#[utoipa::path(
    put,
    path = "/api/auth/role/{id}",
    params(
        ("id" = i32, Path, description = "User ID to update role for")
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 204, description = "Role updated successfully"),
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
) -> Result<StatusCode, ClientApiError> {
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

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/auth/verify",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verified successfully", body = Response<VerifyEmailResponse>),
        (status = 400, description = "Invalid or expired verification code"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth"
)]
pub async fn verify_email(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<VerifyEmailRequest>, JsonRejection>,
) -> Result<Json<Response<VerifyEmailResponse>>, ClientApiError> {
    let Json(payload) = payload?;

    let mut auth_client = AuthServiceClient::connect(state.auth_service_url.clone())
        .await
        .map_err(|e| {
            ClientApiError::InternalError(format!("Failed to connect to auth service: {}", e))
        })?;

    let user_id_request = auth::GetUserIdByEmailRequest {
        email: payload.email.clone(),
    };

    let user_id_response = match auth_client.get_user_id_by_email(user_id_request).await {
        Ok(response) => response.into_inner(),
        Err(status) => {
            return Err(match status.code() {
                tonic::Code::NotFound => ClientApiError::NotFound(status.message().to_string()),
                _ => ClientApiError::InternalError(status.message().to_string()),
            });
        }
    };

    let mut email_client = EmailServiceClient::connect(state.email_service_url.clone())
        .await
        .map_err(|e| {
            ClientApiError::InternalError(format!("Failed to connect to email service: {}", e))
        })?;

    let verify_request = VerifyCodeRequest {
        user_id: user_id_response.user_id,
        verification_code: payload.verification_code,
    };

    let response = match email_client.verify_code(verify_request).await {
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
        return Err(ClientApiError::BadRequest(response.message));
    }

    let mark_verified_request = auth::MarkEmailVerifiedRequest {
        user_id: user_id_response.user_id,
    };

    match auth_client.mark_email_verified(mark_verified_request).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to mark email as verified in auth service: {}", e);
        }
    }

    let verify_response = VerifyEmailResponse {
        success: true,
        message: "Email verified successfully".to_string(),
    };

    Ok(Json(build_verify_email_response(verify_response, &state.base_url)))
}

#[utoipa::path(
    post,
    path = "/api/auth/resend",
    request_body = ResendVerificationRequest,
    responses(
        (status = 200, description = "Verification code resent", body = Response<ResendVerificationResponse>),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth"
)]
pub async fn resend_verification(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<ResendVerificationRequest>, JsonRejection>,
) -> Result<Json<Response<ResendVerificationResponse>>, ClientApiError> {
    let Json(payload) = payload?;

    let mut auth_client = AuthServiceClient::connect(state.auth_service_url.clone())
        .await
        .map_err(|e| {
            ClientApiError::InternalError(format!("Failed to connect to auth service: {}", e))
        })?;

    let user_id_request = auth::GetUserIdByEmailRequest {
        email: payload.email.clone(),
    };

    let user_id_response = match auth_client.get_user_id_by_email(user_id_request).await {
        Ok(response) => response.into_inner(),
        Err(status) => {
            return Err(match status.code() {
                tonic::Code::NotFound => ClientApiError::NotFound(status.message().to_string()),
                _ => ClientApiError::InternalError(status.message().to_string()),
            });
        }
    };

    let mut email_client = EmailServiceClient::connect(state.email_service_url.clone())
        .await
        .map_err(|e| {
            ClientApiError::InternalError(format!("Failed to connect to email service: {}", e))
        })?;

    let resend_request = EmailResendRequest {
        user_id: user_id_response.user_id,
        email: payload.email,
    };

    let response = match email_client.resend_verification_code(resend_request).await {
        Ok(response) => response.into_inner(),
        Err(status) => {
            return Err(match status.code() {
                tonic::Code::NotFound => ClientApiError::NotFound(status.message().to_string()),
                _ => ClientApiError::InternalError(status.message().to_string()),
            });
        }
    };

    let resend_response = ResendVerificationResponse {
        success: response.success,
        message: response.message,
    };

    Ok(Json(build_resend_verification_response(resend_response, &state.base_url)))
}
