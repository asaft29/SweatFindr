use crate::repositories::client_repo::ClientRepo;
use std::sync::Arc;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;
pub mod utils;

pub struct AppState {
    pub client_repo: Arc<ClientRepo>,
    pub base_url: String,
    pub event_service_url: String,
}
