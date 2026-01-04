use anyhow::Result;
use axum::middleware;
use axum::{Router, extract::State, routing::get};
use common::rabbitmq::RabbitMQ;
use event_service::middleware::auth::auth_middleware;
use event_service::services::refund_consumer::RefundRequestConsumer;
use event_service::{
    AppState, handlers,
    repositories::{
        event_packets_repo::EventPacketRepo, event_repo::EventRepo, join_pe_repo::JoinPeRepo,
        refund_repo::RefundRepo, ticket_repo::TicketRepo,
    },
};
use sqlx::postgres::PgPoolOptions;
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
        .compact()
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var is not set!");

    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&db_url)
        .await?;

    info!("{:<12} - Database connection pool created.", "DB");

    let rabbitmq_url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://admin:password@rabbitmq:5672".to_string());

    let rabbitmq = Arc::new(RabbitMQ::new());
    if let Err(e) = rabbitmq.connect(&rabbitmq_url).await {
        warn!(
            "Failed to connect to RabbitMQ: {:?}. Refund system will not work.",
            e
        );
    } else {
        info!("{:<12} - RabbitMQ connection established.", "RABBITMQ");
    }

    let auth_service_url = std::env::var("AUTH_SERVICE_URL")
        .unwrap_or_else(|_| "http://auth-service:50051".to_string());

    let refund_repo = Arc::new(RefundRepo::new(pool.clone()));

    let app_state = Arc::new(AppState {
        event_repo: Arc::new(EventRepo::new(pool.clone())),
        event_packet_repo: Arc::new(EventPacketRepo::new(pool.clone())),
        ticket_repo: Arc::new(TicketRepo::new(pool.clone())),
        join_repo: Arc::new(JoinPeRepo::new(pool.clone())),
        refund_repo: Arc::clone(&refund_repo),
        rabbitmq: Arc::clone(&rabbitmq),
        base_url: "http://localhost:8001/api/event-manager".to_string(),
        auth_service_url,
    });

    let consumer_rabbitmq = Arc::clone(&rabbitmq);
    let consumer_refund_repo = Arc::clone(&refund_repo);
    tokio::spawn(async move {
        let consumer = RefundRequestConsumer::new(consumer_rabbitmq, consumer_refund_repo);
        if let Err(e) = consumer.start().await {
            error!("Refund request consumer error: {:?}", e);
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api", get(check_state))
        .nest(
            "/api/event-manager",
            handlers::authenticated_api_router().layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest("/api/event-manager", handlers::public_api_router())
        .merge(handlers::swagger_router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());

    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_state(State(status): State<Arc<AppState>>) -> &'static str {
    match status.event_repo.check().await {
        Ok(_) => "PostgreSQL works! :p",
        Err(_) => "PostgreSQL DOESN'T work! :(",
    }
}
