pub mod auth_service;
pub mod event_manager;
pub mod event_service {
    pub use super::event_manager::*;
    pub type EventServiceError = super::event_manager::ExternalServiceError;
}
