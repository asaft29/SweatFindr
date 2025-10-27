use anyhow::Result;
use axum::{Router, extract::State, routing::get};
use event_service::{
    AppState, handlers,
    repositories::{event_packets_repo::EventPacketRepo, event_repo::EventRepo},
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
        .max_connections(5)
        .connect(&db_url)
        .await?;

    info!("{:<12} - Database connection pool created.", "DB");

    let app_state = Arc::new(AppState {
        event_repo: Arc::new(EventRepo::new(pool.clone())),
        event_packet_repo: Arc::new(EventPacketRepo::new(pool.clone())),
        base_url: "/api/event-manager".to_string(),
    });

    // Correct router setup
    let app = Router::new()
        .route("/api", get(check_state))
        .nest("/api/event-manager", handlers::api_router())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());

    // Use axum::serve
    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_state(State(status): State<Arc<AppState>>) -> &'static str {
    match status.event_repo.check().await {
        Ok(_) => "PostgreSQL works! :p",
        Err(_) => "PostgreSQL DOESN'T work! :(",
    }
}
