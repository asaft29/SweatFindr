use crate::AppState;
use crate::error::JoinPeRepoError;
use crate::links::{Response, build_event_over_packet, build_packet_over_event};
use crate::models::event::Event;
use crate::models::event_packets::EventPackets;
use crate::models::join_pe::{AddEventToPacket, AddPacketToEvent};
use axum::Router;
use axum::response::IntoResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;

pub async fn get_packets_for_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, JoinPeRepoError> {
    let packets = state.join_repo.get_packets_for_event(id).await?;

    let wrapped: Vec<Response<EventPackets>> = packets
        .into_iter()
        .map(|e| build_packet_over_event(e, id, &state.base_url))
        .collect();

    Ok(Json(wrapped))
}

pub async fn get_events_for_packet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, JoinPeRepoError> {
    let events = state.join_repo.get_events_for_packet(id).await?;

    let wrapped: Vec<Response<Event>> = events
        .into_iter()
        .map(|e| build_event_over_packet(e, id, &state.base_url))
        .collect();

    Ok(Json(wrapped))
}

pub async fn add_event_to_packet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<AddEventToPacket>,
) -> Result<impl IntoResponse, JoinPeRepoError> {
    let relation = state.join_repo.add_event_to_packet(id, payload).await?;
    Ok((StatusCode::CREATED, Json(relation)))
}

pub async fn add_packet_to_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<AddPacketToEvent>,
) -> Result<impl IntoResponse, JoinPeRepoError> {
    let relation = state.join_repo.add_packet_to_event(id, payload).await?;
    Ok((StatusCode::CREATED, Json(relation)))
}

pub fn join_pe_manager_router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/events/{id}/event-packets",
            get(get_packets_for_event).post(add_packet_to_event),
        )
        .route(
            "/event-packets/{id}/events",
            get(get_events_for_packet).post(add_event_to_packet),
        )
}
