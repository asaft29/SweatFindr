use crate::AppState;
use crate::auth::auth_service_client::AuthServiceClient;
use crate::auth::{
    AuthRequest, AuthResponse, DestroyRequest, DestroyResponse, RegisterRequest, RegisterResponse,
};
use crate::gateway::map_grpc_error;
use crate::middleware::auth::auth_middleware;
use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header::AUTHORIZATION, HeaderMap},
    middleware,
    routing::post,
};
use std::sync::Arc;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route(
            "/logout",
            post(logout).layer(middleware::from_fn_with_state(state, auth_middleware)),
        )
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let mut client = AuthServiceClient::new(state.auth_channel.clone());

    match client.register_user(request).await {
        Ok(response) => Ok(Json(response.into_inner())),
        Err(e) => Err(map_grpc_error(e)),
    }
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let mut client = AuthServiceClient::new(state.auth_channel.clone());

    match client.authenticate(request).await {
        Ok(response) => Ok(Json(response.into_inner())),
        Err(e) => Err(map_grpc_error(e)),
    }
}

async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<DestroyResponse>, StatusCode> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let request = DestroyRequest {
        token_value: token.to_string(),
    };

    let mut client = AuthServiceClient::new(state.auth_channel.clone());

    match client.destroy_token(request).await {
        Ok(response) => Ok(Json(response.into_inner())),
        Err(e) => Err(map_grpc_error(e)),
    }
}
