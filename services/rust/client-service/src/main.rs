use anyhow::Result;
use axum::{Router, extract::State, routing::get};
use client_service::services::event_manager::EventManagerClient;
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

    let auth_service_url = std::env::var("AUTH_SERVICE_URL")
        .unwrap_or_else(|_| "http://auth-service:50051".to_string());

    let service_username = std::env::var("SERVICE_USERNAME").expect("SERVICE_USERNAME must be set");
    let service_password = std::env::var("SERVICE_PASSWORD").expect("SERVICE_PASSWORD must be set");

    info!("{:<12} - Authenticating as service user...", "AUTH");

    use client_service::handlers::auth::auth::AuthRequest;
    use client_service::handlers::auth::auth::auth_service_client::AuthServiceClient;

    let mut auth_client = AuthServiceClient::connect(auth_service_url.clone())
        .await
        .expect("Failed to connect to auth service for service authentication");

    let grpc_request = AuthRequest {
        username: service_username,
        password: service_password,
    };

    let auth_response = auth_client
        .authenticate(grpc_request)
        .await
        .expect("Failed to authenticate service user")
        .into_inner();

    if !auth_response.success {
        panic!("Service authentication failed: {}", auth_response.message);
    }

    let service_token = auth_response.token_value;
    info!("{:<12} - Service authentication successful", "AUTH");

    let event_manager_client = Arc::new(EventManagerClient::new(event_service_url.clone()));

    let app_state = Arc::new(AppState {
        client_repo: Arc::new(ClientRepo::new(database)),
        base_url: "http://localhost:8002/api/client-manager".to_string(),
        event_service_url,
        auth_service_url,
        service_token,
        event_manager_client,
    });

    use axum::middleware;
    use client_service::middleware::auth::auth_middleware;

    let app = Router::new()
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
