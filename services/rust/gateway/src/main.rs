mod gateway;
mod handlers;
mod middleware;

use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use tonic::transport::Channel;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

pub mod auth {
    tonic::include_proto!("auth");
}

pub mod email {
    tonic::include_proto!("email");
}

#[derive(Clone)]
pub struct AppState {
    pub auth_channel: Channel,
    pub email_channel: Channel,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .compact()
        .init();

    let auth_service_url = std::env::var("AUTH_SERVICE_URL")
        .unwrap_or_else(|_| "http://auth-service:50051".to_string());

    let email_service_url = std::env::var("EMAIL_SERVICE_URL")
        .unwrap_or_else(|_| "http://email-service:50052".to_string());

    let auth_channel = Channel::from_shared(auth_service_url)?.connect().await?;
    let email_channel = Channel::from_shared(email_service_url)?.connect().await?;

    let app_state = Arc::new(AppState {
        auth_channel,
        email_channel,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api/auth", handlers::auth::router())
        .nest("/api/email", handlers::email::router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "10000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(
        "{:<12} - gRPC Gateway listening on {:?}",
        "LISTENING",
        listener.local_addr()
    );
    info!(
        "{:<12} - Proxying auth-service and email-service gRPC calls",
        "INFO"
    );

    axum::serve(listener, app).await?;

    Ok(())
}
