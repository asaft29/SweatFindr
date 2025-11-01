use anyhow::Result;
use axum::{Router, extract::State, routing::get};
use event_service::{
    AppState, handlers,
    repositories::{
        event_packets_repo::EventPacketRepo, event_repo::EventRepo, join_pe_repo::JoinPeRepo,
        ticket_repo::TicketRepo,
    },
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{Level, info};
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .compact()
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var is not set!");

    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&db_url)
        .await?;

    info!("{:<12} - Database connection pool created.", "DB");

    let app_state = Arc::new(AppState {
        event_repo: Arc::new(EventRepo::new(pool.clone())),
        event_packet_repo: Arc::new(EventPacketRepo::new(pool.clone())),
        ticket_repo: Arc::new(TicketRepo::new(pool.clone())),
        join_repo: Arc::new(JoinPeRepo::new(pool.clone())),
        base_url: "http://localhost:8001/api/event-manager".to_string(),
    });

    let app = Router::new()
        .route("/api", get(check_state))
        .nest("/api/event-manager", handlers::api_router())
        .merge(handlers::swagger_router())
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
