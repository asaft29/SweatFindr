use crate::AppState;
use crate::error::EventPacketRepoError;
use crate::links::EventPacketResponse;
use crate::models::event_packets::{CreateEventPacket, UpdateEventPacket};
use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;

pub async fn get_event_packet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<EventPacketResponse>, EventPacketRepoError> {
    // Use the packet_repo
    let event_packet = state.event_packet_repo.get_event_packet(id).await?;

    // Use the EventPacketResponse
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
    Json(payload): Json<CreateEventPacket>, // Use CreateEventPacket
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
        .route("/event-packets", post(create_event_packet))
        .route(
            "/event-packets/{id}",
            get(get_event_packet)
                .put(update_event_packet)
                .delete(delete_event_packet),
        )
}
