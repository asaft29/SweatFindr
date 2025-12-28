use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::error;

use crate::{AppState, auth::auth_service_client::AuthServiceClient};

#[derive(Clone)]
#[allow(dead_code)]
pub struct AuthUser {
    pub user_id: i32,
    pub role: String,
}

pub async fn auth_middleware(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let state = request
        .extensions()
        .get::<Arc<AppState>>()
        .ok_or_else(|| {
            error!("AppState not found in request extensions");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .clone();

    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header_value) => {
            if let Some(token) = header_value.strip_prefix("Bearer ") {
                token
            } else {
                error!("Invalid Authorization header format");
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        _ => {
            error!("Missing Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let mut client = AuthServiceClient::new(state.auth_channel.clone());

    let grpc_request = crate::auth::ValidateRequest {
        token_value: token.to_string(),
    };

    let response = client.validate_token(grpc_request).await.map_err(|e| {
        error!("gRPC error during token validation: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let validate_response = response.into_inner();

    if !validate_response.valid {
        error!("Token validation failed: {}", validate_response.message);
        return Err(StatusCode::UNAUTHORIZED);
    }

    let auth_user = AuthUser {
        user_id: validate_response.user_id,
        role: validate_response.role,
    };

    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}
