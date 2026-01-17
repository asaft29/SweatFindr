mod middleware;
mod websocket;

use axum::{Router, middleware as axum_middleware, routing::get};
use axum_prometheus::PrometheusMetricLayer;
use common::rabbitmq::RabbitMQ;
use std::sync::Arc;
use tonic::transport::Channel;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use websocket::{ConnectionManager, WebSocketBroker};

pub mod auth {
    tonic::include_proto!("auth");
}

#[derive(Clone)]
pub struct AppState {
    pub ws_manager: Arc<ConnectionManager>,
    pub auth_channel: Channel,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    let auth_service_url = std::env::var("AUTH_SERVICE_URL")
        .unwrap_or_else(|_| "http://auth-service:50051".to_string());
    let auth_channel = Channel::from_shared(auth_service_url)?.connect().await?;

    let ws_manager = Arc::new(ConnectionManager::new());

    let rabbitmq_url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://admin:password@rabbitmq:5672".to_string());
    let rabbitmq = Arc::new(RabbitMQ::new());

    if let Err(e) = rabbitmq.connect(&rabbitmq_url).await {
        warn!(
            "Failed to connect to RabbitMQ: {:?}. WebSocket updates disabled.",
            e
        );
    } else {
        info!("RabbitMQ connection established");
        let broker = WebSocketBroker::new(Arc::clone(&rabbitmq), Arc::clone(&ws_manager));
        tokio::spawn(async move {
            if let Err(e) = broker.start().await {
                error!("WebSocket broker error: {:?}", e);
            }
        });
    }

    let app_state = Arc::new(AppState {
        ws_manager,
        auth_channel,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app = Router::new()
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .route(
            "/ws",
            get(websocket::websocket_handler).layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                middleware::auth::ws_auth_middleware,
            )),
        )
        .layer(prometheus_layer)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(
        "{:<12} - Notification service listening on {:?}",
        "LISTENING",
        listener.local_addr()
    );

    axum::serve(listener, app).await?;

    Ok(())
}
