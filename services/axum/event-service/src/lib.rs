pub mod handlers;
pub mod links;
pub mod models;
pub mod repositories;

use crate::repositories::event_repo::EventRepo;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<EventRepo>,
    pub base_url: String,
}
