use anyhow::Result;
use axum::{Router, extract::State, routing::get};
use client_service::{AppState, handlers, repositories::client_repo::ClientRepo};
use mongodb::{Client, options::ClientOptions};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .compact()
        .init();

    let mongodb_uri = std::env::var("MONGODB_URI").expect("MONGODB_URI env var is not set!");

    let client_options = ClientOptions::parse(&mongodb_uri).await?;
    let client = Client::with_options(client_options)?;

    let database = client.database("clientsdb");

    info!("{:<12} - MongoDB connection established.", "DB");

    let event_service_url = std::env::var("EVENT_SERVICE_URL")
        .unwrap_or_else(|_| "http://event-service:8080".to_string());

    let app_state = Arc::new(AppState {
        client_repo: Arc::new(ClientRepo::new(database)),
        base_url: "http://localhost:8002/api/client-manager".to_string(),
        event_service_url,
    });

    let app = Router::new()
        .route("/api", get(check_state))
        .nest("/api/client-manager", handlers::api_router())
        .merge(handlers::swagger_router())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());

    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_state(State(state): State<Arc<AppState>>) -> &'static str {
    match state.client_repo.check().await {
        Ok(_) => "MongoDB works! :)",
        Err(_) => "MongoDB DOESN'T work! :(",
    }
}
