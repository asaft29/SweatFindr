mod grpc_service;
mod repository;
mod services;

use crate::grpc_service::email::email_service_server::EmailServiceServer;
use crate::grpc_service::EmailServiceImpl;
use crate::repository::verification_repository::VerificationRepository;
use crate::services::email_service::EmailService;
use crate::services::expiration_listener::ExpirationListener;
use anyhow::Result;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::{error, info};

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

    let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let server_port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "50052".to_string())
        .parse::<u16>()?;

    let addr = format!("{}:{}", server_host, server_port).parse()?;

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_client = redis::Client::open(redis_url.as_str())?;
    let redis_conn = redis::aio::ConnectionManager::new(redis_client).await?;

    let verification_repo = Arc::new(VerificationRepository::new(redis_conn));
    let email_service = EmailService::new()?;

    let auth_service_url = std::env::var("AUTH_SERVICE_URL")
        .unwrap_or_else(|_| "http://auth-service:50051".to_string());

    let expiration_listener = Arc::new(ExpirationListener::new(auth_service_url.clone()));
    let redis_url_clone = redis_url.clone();
    let expiration_task = tokio::spawn(async move {
        if let Err(e) = expiration_listener.start(redis_url_clone).await {
            error!(error = %e, "Expiration listener error");
        }
    });

    let service = EmailServiceImpl::new(email_service, verification_repo, auth_service_url);

    info!("Email service listening on {}", addr);
    info!("Redis expiration listener started (event-driven cleanup)");

    tokio::select! {
        result = Server::builder()
            .add_service(EmailServiceServer::new(service))
            .serve(addr) => {
                result?;
            }
        _ = expiration_task => {
            error!("Expiration listener ended unexpectedly");
        }
    }

    Ok(())
}
