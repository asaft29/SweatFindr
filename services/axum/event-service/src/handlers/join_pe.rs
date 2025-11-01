use crate::AppState;
use crate::models::event::Event;
use crate::models::event_packets::EventPackets;
use crate::models::join_pe::{AddEventToPacket, AddPacketToEvent};
use crate::shared::error::JoinPeRepoError;
use crate::shared::links::{Response, build_event_over_packet, build_packet_over_event};
use axum::Router;
use axum::response::IntoResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/event-manager/events/{id}/event-packets",
    params(
        ("id" = i32, Path, description = "ID of the event")
    ),
    responses(
        (status = 200, description = "List packets linked to the specified event", body = [Response<EventPackets>]),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "JoinPE"
)]
pub async fn list_packets_for_event(
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

#[utoipa::path(
    get,
    path = "/api/event-manager/event-packets/{id}/events",
    params(
        ("id" = i32, Path, description = "ID of the event packet")
    ),
    responses(
        (status = 200, description = "List events linked to the specified event packet", body = [Response<Event>]),
        (status = 404, description = "Event packet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "JoinPE"
)]
pub async fn list_events_for_packet(
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

#[utoipa::path(
    post,
    path = "/api/event-manager/event-packets/{id}/events",
    request_body = AddEventToPacket,
    params(
        ("id" = i32, Path, description = "ID of the event packet")
    ),
    responses(
        (status = 201, description = "Event successfully linked to event packet"),
        (status = 404, description = "Event or packet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "JoinPE"
)]
pub async fn add_event_to_packet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<AddEventToPacket>,
) -> Result<impl IntoResponse, JoinPeRepoError> {
    let relation = state.join_repo.add_event_to_packet(id, payload).await?;
    Ok((StatusCode::CREATED, Json(relation)))
}

#[utoipa::path(
    post,
    path = "/api/event-manager/events/{id}/event-packets",
    request_body = AddPacketToEvent,
    params(
        ("id" = i32, Path, description = "ID of the event")
    ),
    responses(
        (status = 201, description = "Event packet successfully linked to event"),
        (status = 404, description = "Event or packet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "JoinPE"
)]
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
            get(list_packets_for_event).post(add_packet_to_event),
        )
        .route(
            "/event-packets/{id}/events",
            get(list_events_for_packet).post(add_event_to_packet),
        )
}
