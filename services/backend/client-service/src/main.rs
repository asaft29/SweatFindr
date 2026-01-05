use anyhow::Result;
use axum::middleware;
use axum::{Router, extract::State, routing::get};
use axum_prometheus::PrometheusMetricLayer;
use client_service::handlers::auth::auth::AuthRequest;
use client_service::handlers::auth::auth::auth_service_client::AuthServiceClient;
use client_service::middleware::auth::auth_middleware;
use client_service::services::RefundConsumer;
use client_service::services::event_manager::EventManagerClient;
use client_service::{AppState, handlers, repositories::client_repo::ClientRepo};
use common::rabbitmq::RabbitMQ;
use mongodb::{Client, options::ClientOptions};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    let mongodb_uri = std::env::var("MONGODB_URI").expect("MONGODB_URI env var is not set!");

    let client_options = ClientOptions::parse(&mongodb_uri).await?;
    let client = Client::with_options(client_options)?;

    let database = client.database("clientsdb");

    info!("{:<12} - MongoDB connection established.", "DB");

    let event_service_url = std::env::var("EVENT_SERVICE_URL")
        .unwrap_or_else(|_| "http://event-service:8080".to_string());

    let auth_service_url = std::env::var("AUTH_SERVICE_URL")
        .unwrap_or_else(|_| "http://auth-service:50051".to_string());

    let email_service_url = std::env::var("EMAIL_SERVICE_URL")
        .unwrap_or_else(|_| "http://email-service:50052".to_string());

    let rabbitmq_url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://admin:password@rabbitmq:5672".to_string());

    let rabbitmq = Arc::new(RabbitMQ::new());
    if let Err(e) = rabbitmq.connect(&rabbitmq_url).await {
        warn!(
            "Failed to connect to RabbitMQ: {:?}. Refund requests will not work.",
            e
        );
    } else {
        info!("{:<12} - RabbitMQ connection established.", "RABBITMQ");
    }

    let service_username = std::env::var("SERVICE_USERNAME").expect("SERVICE_USERNAME must be set");
    let service_password = std::env::var("SERVICE_PASSWORD").expect("SERVICE_PASSWORD must be set");

    info!("{:<12} - Authenticating as service user...", "AUTH");

    let mut auth_client = AuthServiceClient::connect(auth_service_url.clone())
        .await
        .expect("Failed to connect to auth service for service authentication");

    let auth_response = auth_client
        .authenticate(tonic::Request::new(AuthRequest {
            username: service_username,
            password: service_password,
        }))
        .await
        .expect("Failed to authenticate service user")
        .into_inner();

    if !auth_response.success {
        panic!("Service authentication failed: {}", auth_response.message);
    }

    let service_token = auth_response.token_value;
    info!("{:<12} - Service authentication successful", "AUTH");

    let event_manager_client = Arc::new(EventManagerClient::new(event_service_url.clone()));

    let client_repo = Arc::new(ClientRepo::new(database));

    let refund_consumer = RefundConsumer::new(Arc::clone(&rabbitmq), Arc::clone(&client_repo));
    let refund_consumer_task = tokio::spawn(async move {
        if let Err(e) = refund_consumer.start().await {
            error!("Refund consumer error: {:?}", e);
        }
    });

    let app_state = Arc::new(AppState {
        client_repo,
        base_url: "http://localhost:8002/api/client-manager".to_string(),
        event_service_url,
        auth_service_url,
        email_service_url,
        service_token,
        event_manager_client,
        rabbitmq,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app = Router::new()
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .route("/api", get(check_state))
        .nest("/api/auth", handlers::auth_router())
        .nest(
            "/api/auth",
            handlers::auth_protected_router().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest(
            "/api/client-manager",
            handlers::api_router().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .merge(handlers::swagger_router())
        .layer(prometheus_layer)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());

    tokio::select! {
        result = axum::serve(listener, app) => {
            result?;
        }
        _ = refund_consumer_task => {
            error!("Refund consumer ended unexpectedly");
        }
    }

    Ok(())
}

async fn check_state(State(state): State<Arc<AppState>>) -> &'static str {
    match state.client_repo.check().await {
        Ok(_) => "MongoDB works! :)",
        Err(_) => "MongoDB DOESN'T work! :(",
    }
}
