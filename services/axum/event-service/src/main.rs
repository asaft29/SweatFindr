use std::sync::Arc;

use anyhow::Result;
use axum::{Router, extract::State, routing::get};
use event_service::{handlers, repositories::event_repo::EventRepo};
use sqlx::postgres::PgPoolOptions;
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

    let repo = Arc::new(EventRepo::new(pool.clone()));

    // Correct router setup
    let app = Router::new()
        .route("/api", get(check_state))
        .nest(
            "/api/event-manager",
            handlers::routes::event_manager_router(),
        )
        .layer(TraceLayer::new_for_http()) // top-level layer
        .with_state(repo); // top-level state

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());

    // Use axum::serve
    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_state(State(repo): State<Arc<EventRepo>>) -> &'static str {
    match repo.check().await {
        Ok(_) => "PostgreSQL works! :p",
        Err(_) => "PostgreSQL DOESN'T work! :(",
    }
}
