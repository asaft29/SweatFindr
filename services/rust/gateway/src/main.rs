mod handlers;
mod middleware;
mod models;

use anyhow::Result;
use axum::Router;
use std::sync::Arc;
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
    pub auth_service_url: String,
    pub email_service_url: String,
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

    let app_state = Arc::new(AppState {
        auth_service_url,
        email_service_url,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api/gateway/auth", handlers::auth::router())
        .nest("/api/gateway/email", handlers::email::router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "10000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("{:<12} - gRPC Gateway listening on {:?}", "LISTENING", listener.local_addr());
    info!("{:<12} - Proxying auth-service and email-service gRPC calls", "INFO");

    axum::serve(listener, app).await?;

    Ok(())
}
