pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod utils;

use crate::repositories::event_packets_repo::EventPacketRepo;
use crate::repositories::event_repo::EventRepo;
use crate::repositories::join_pe_repo::JoinPeRepo;
use crate::repositories::ticket_repo::TicketRepo;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub event_repo: Arc<EventRepo>,
    pub event_packet_repo: Arc<EventPacketRepo>,
    pub ticket_repo: Arc<TicketRepo>,
    pub join_repo: Arc<JoinPeRepo>,
    pub base_url: String,
    pub auth_service_url: String,
}
