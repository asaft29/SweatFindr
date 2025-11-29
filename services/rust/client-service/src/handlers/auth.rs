use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::AppState;
use crate::models::client::{CreateClient, SocialMedia};
use crate::utils::error::ClientApiError;

pub mod auth {
    tonic::include_proto!("auth");
}

use auth::RegisterRequest as GrpcRegisterRequest;
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
    Router::new().route("/register", post(register))
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
            message: "User registered successfully".to_string(),
            client_id: Some(created_client.id.to_hex()),
        }),
    ))
}
