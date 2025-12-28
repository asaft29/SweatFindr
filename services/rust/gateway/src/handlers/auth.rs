use crate::AppState;
use crate::auth::auth_service_client::AuthServiceClient;
use crate::auth::{
    AuthRequest, AuthResponse, DestroyRequest, DestroyResponse, RegisterRequest, RegisterResponse,
};
use crate::gateway::map_grpc_error;
use crate::middleware::auth::auth_middleware;
use axum::{Json, Router, extract::State, http::StatusCode, middleware, routing::post};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route(
            "/logout",
            post(logout).layer(middleware::from_fn(auth_middleware)),
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
    Json(request): Json<DestroyRequest>,
) -> Result<Json<DestroyResponse>, StatusCode> {
    let mut client = AuthServiceClient::new(state.auth_channel.clone());

    match client.destroy_token(request).await {
        Ok(response) => Ok(Json(response.into_inner())),
        Err(e) => Err(map_grpc_error(e)),
    }
}
