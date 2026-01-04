mod grpc_service;
mod repository;
mod services;

use crate::grpc_service::email::email_service_server::EmailServiceServer;
use crate::grpc_service::EmailServiceImpl;
use crate::repository::verification_repository::VerificationRepository;
use crate::services::email_service::EmailService;
use crate::services::refund_consumer::RefundConsumer;
use anyhow::Result;
use common::rabbitmq::RabbitMQ;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::{error, info, warn};

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
    let email_service = Arc::new(EmailService::new()?);

    let auth_service_url = std::env::var("AUTH_SERVICE_URL")
        .unwrap_or_else(|_| "http://auth-service:50051".to_string());

    let rabbitmq_url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://admin:password@rabbitmq:5672".to_string());

    let rabbitmq = Arc::new(RabbitMQ::new());

    let refund_consumer_task = {
        let rabbitmq = Arc::clone(&rabbitmq);
        let email_service = Arc::clone(&email_service);
        tokio::spawn(async move {
            if let Err(e) = rabbitmq.connect(&rabbitmq_url).await {
                warn!(
                    "Failed to connect to RabbitMQ: {:?}. Refund notifications disabled.",
                    e
                );
                return;
            }
            info!("RabbitMQ connection established for refund consumer");

            let consumer = RefundConsumer::new(rabbitmq, email_service);
            if let Err(e) = consumer.start().await {
                error!("Refund consumer error: {:?}", e);
            }
        })
    };

    let service = EmailServiceImpl::new(
        Arc::clone(&email_service),
        verification_repo,
        auth_service_url,
    );

    info!("Email service listening on {}", addr);
    info!("RabbitMQ refund consumer started");

    tokio::select! {
        result = Server::builder()
            .add_service(EmailServiceServer::new(service))
            .serve(addr) => {
                result?;
            }
        _ = refund_consumer_task => {
            warn!("Refund consumer ended");
        }
    }

    Ok(())
}
