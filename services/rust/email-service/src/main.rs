mod grpc_service;
mod repository;
mod services;

use crate::grpc_service::email::email_service_server::EmailServiceServer;
use crate::grpc_service::EmailServiceImpl;
use crate::repository::verification_repository::VerificationRepository;
use crate::services::email_service::EmailService;
use anyhow::Result;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let server_port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "50052".to_string())
        .parse::<u16>()?;

    let addr = format!("{}:{}", server_host, server_port).parse()?;

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_client = redis::Client::open(redis_url.as_str())?;
    let redis_conn = redis::aio::ConnectionManager::new(redis_client).await?;

    let verification_repo = VerificationRepository::new(redis_conn);
    let email_service = EmailService::new()?;

    let service = EmailServiceImpl::new(email_service, verification_repo);

    println!("Email service listening on {}", addr);

    Server::builder()
        .add_service(EmailServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
