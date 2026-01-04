use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::sync::Arc;

use crate::AppState;
use super::UserClaims;

pub mod middleware {
    tonic::include_proto!("auth");
}

use middleware::ValidateRequest;
use middleware::auth_service_client::AuthServiceClient;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Missing Authorization header"
                })),
            )
        })?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Invalid Authorization header format. Expected: Bearer <token>"
                })),
            )
        })?
        .to_string();

    let mut client = AuthServiceClient::connect(state.auth_service_url.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to connect to auth service: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Authentication service unavailable"
                })),
            )
        })?;

    let request_payload = tonic::Request::new(ValidateRequest { token_value: token });

    let response = client
        .validate_token(request_payload)
        .await
        .map_err(|e| {
            tracing::error!("Auth service error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Authentication service error"
                })),
            )
        })?
        .into_inner();

    if !response.success || !response.valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": response.message
            })),
        ));
    }

    let claims = UserClaims {
        user_id: response.user_id,
        role: response.role,
    };

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
