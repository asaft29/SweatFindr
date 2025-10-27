pub mod event_packets;
pub mod events;

use crate::AppState;
use crate::handlers::event_packets::event_packet_manager_router;
use crate::handlers::events::event_manager_router;
use axum::Router;
use std::sync::Arc;

pub fn api_router() -> Router<Arc<AppState>> {
    Router::new()
        .merge(event_manager_router())
        .merge(event_packet_manager_router())
}
