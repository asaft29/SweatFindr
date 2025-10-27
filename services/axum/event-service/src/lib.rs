pub mod error;
pub mod handlers;
pub mod links;
pub mod models;
pub mod repositories;

use crate::repositories::event_packets_repo::EventPacketRepo;
use crate::repositories::event_repo::EventRepo;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub event_repo: Arc<EventRepo>,
    pub event_packet_repo: Arc<EventPacketRepo>,
    pub base_url: String,
}
