use crate::AppState;
use crate::error::EventPacketRepoError;
use crate::links::EventPacketResponse;
use crate::models::event_packets::{CreateEventPacket, EventPacketQuery, UpdateEventPacket};
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;

pub async fn get_event_packets(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EventPacketQuery>,
) -> Result<impl IntoResponse, EventPacketRepoError> {
    let event_packets = state.event_packet_repo.list_event_packets(params).await?;

    let wrapped: Vec<EventPacketResponse> = event_packets
        .into_iter()
        .map(|e| EventPacketResponse::new(e, &state.base_url))
        .collect();

    Ok(Json(wrapped))
}

pub async fn get_event_packet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<EventPacketResponse>, EventPacketRepoError> {
    let event_packet = state.event_packet_repo.get_event_packet(id).await?;

    let packet_response = EventPacketResponse::new(event_packet, &state.base_url);
    Ok(Json(packet_response))
}

pub async fn update_event_packet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateEventPacket>,
) -> Result<Json<EventPacketResponse>, EventPacketRepoError> {
    let event_packet = state
        .event_packet_repo
        .update_event_packet(id, payload)
        .await?;

    let packet_response = EventPacketResponse::new(event_packet, &state.base_url);

    Ok(Json(packet_response))
}

pub async fn create_event_packet(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateEventPacket>,
) -> Result<impl IntoResponse, EventPacketRepoError> {
    let event_packet = state.event_packet_repo.create_event_packet(payload).await?;

    let packet_response = EventPacketResponse::new(event_packet, &state.base_url);

    Ok((StatusCode::CREATED, Json(packet_response)))
}

pub async fn delete_event_packet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, EventPacketRepoError> {
    state.event_packet_repo.delete_event_packet(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn event_packet_manager_router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/event-packets",
            post(create_event_packet).get(get_event_packets),
        )
        .route(
            "/event-packets/{id}",
            get(get_event_packet)
                .put(update_event_packet)
                .delete(delete_event_packet),
        )
}
