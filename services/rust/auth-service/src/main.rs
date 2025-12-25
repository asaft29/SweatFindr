mod grpc_service;
mod models;
mod repository;
mod services;

use anyhow::Result;
use grpc_service::auth::auth_service_server::AuthServiceServer;
use grpc_service::AuthServiceImpl;
use repository::UserRepository;
use services::{JwtService, TokenBlacklist};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let db_host = std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let db_port = std::env::var("DB_PORT").unwrap_or_else(|_| "6000".to_string());
    let db_user = std::env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
    let db_password = std::env::var("DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
    let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "auth_db".to_string());

    let connection_string = format!(
        "host={} port={} user={} password={} dbname={}",
        db_host, db_port, db_user, db_password, db_name
    );

    println!("Connecting to database: {}", connection_string);

    let (client, connection) =
        tokio_postgres::connect(&connection_string, tokio_postgres::tls::NoTls)
            .await
            .expect("Failed to connect to database");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    let user_repo = UserRepository::new(client);
    let jwt_secret = std::env::var("JWT_SECRET").unwrap();
    let jwt_issuer =
        std::env::var("JWT_ISSUER").unwrap_or_else(|_| "http://localhost:50051".to_string());
    let jwt_service = JwtService::new(jwt_secret, jwt_issuer);
    let blacklist = TokenBlacklist::default();

    let email_service_url = std::env::var("EMAIL_SERVICE_URL")
        .unwrap_or_else(|_| "http://email-service:50052".to_string());

    let auth_service = AuthServiceImpl {
        user_repo,
        jwt_service,
        blacklist,
        email_service_url,
    };

    let addr = std::env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50051".to_string())
        .parse()
        .expect("Invalid GRPC_ADDR");

    println!("gRPC Auth Service listening on {}", addr);

    Server::builder()
        .add_service(AuthServiceServer::new(auth_service))
        .serve(addr)
        .await?;

    Ok(())
}
