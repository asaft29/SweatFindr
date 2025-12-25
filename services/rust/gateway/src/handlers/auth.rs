use crate::models::auth::*;
use crate::AppState;
use crate::auth::auth_service_client::AuthServiceClient;
use axum::{extract::State, http::StatusCode, Json, Router, routing::post};
use std::sync::Arc;
use tracing::error;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let mut client = AuthServiceClient::connect(state.auth_service_url.clone())
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to connect to auth service");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    let grpc_request = crate::auth::RegisterRequest {
        email: payload.email,
        password: payload.password,
        role: payload.role,
    };

    match client.register_user(grpc_request).await {
        Ok(response) => {
            let res = response.into_inner();
            Ok(Json(RegisterResponse {
                success: res.success,
                message: res.message,
                user_id: if res.user_id > 0 { Some(res.user_id) } else { None },
                token_value: if !res.token_value.is_empty() { Some(res.token_value) } else { None },
            }))
        }
        Err(e) => {
            error!(error = %e, "gRPC error during registration");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let mut client = AuthServiceClient::connect(state.auth_service_url.clone())
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to connect to auth service");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    let grpc_request = crate::auth::AuthRequest {
        username: payload.username,
        password: payload.password,
    };

    match client.authenticate(grpc_request).await {
        Ok(response) => {
            let res = response.into_inner();
            Ok(Json(LoginResponse {
                success: res.success,
                message: res.message,
                token_value: if !res.token_value.is_empty() { Some(res.token_value) } else { None },
            }))
        }
        Err(e) => {
            error!(error = %e, "gRPC error during login");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

async fn logout(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LogoutRequest>,
) -> Result<Json<LogoutResponse>, StatusCode> {
    let mut client = AuthServiceClient::connect(state.auth_service_url.clone())
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to connect to auth service");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    let grpc_request = crate::auth::DestroyRequest {
        token_value: payload.token_value,
    };

    match client.destroy_token(grpc_request).await {
        Ok(response) => {
            let res = response.into_inner();
            Ok(Json(LogoutResponse {
                success: res.success,
                message: res.message,
            }))
        }
        Err(e) => {
            error!(error = %e, "gRPC error during logout");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
