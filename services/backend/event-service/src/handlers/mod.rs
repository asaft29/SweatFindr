pub mod event;
pub mod event_packets;
pub mod join_pe;
pub mod refund;
pub mod ticket;

use crate::AppState;
use crate::handlers::event::{event_manager_router, public_event_router};
use crate::handlers::event_packets::{event_packet_manager_router, public_event_packet_router};
use crate::handlers::join_pe::{join_pe_manager_router, public_join_pe_router};
use crate::handlers::refund::refund_router;
use crate::handlers::ticket::ticket_manager_router;
use crate::utils::doc::ApiDoc;
use axum::Router;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn authenticated_api_router() -> Router<Arc<AppState>> {
    Router::new()
        .merge(event_manager_router())
        .merge(event_packet_manager_router())
        .merge(ticket_manager_router())
        .merge(join_pe_manager_router())
        .merge(refund_router())
}

pub fn public_api_router() -> Router<Arc<AppState>> {
    Router::new()
        .merge(public_event_router())
        .merge(public_event_packet_router())
        .merge(public_join_pe_router())
}

pub fn swagger_router() -> Router<Arc<AppState>> {
    Router::new().merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", ApiDoc::openapi()))
}
