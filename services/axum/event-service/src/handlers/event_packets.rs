use crate::AppState;
use crate::error::{EventPacketRepoError, TicketRepoError};
use crate::links::{
    Response, build_filtered_event_packets, build_simple_event_packet, build_ticket_over_packet,
};
use crate::models::event_packets::{
    CreateEventPacket, EventPacketQuery, EventPackets, UpdateEventPacket,
};
use crate::models::ticket::{CreateTicket, Ticket, UpdateTicket};
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;

pub async fn list_event_packets(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EventPacketQuery>,
) -> Result<impl IntoResponse, EventPacketRepoError> {
    let event_packets = state
        .event_packet_repo
        .list_event_packets(params.clone())
        .await?;

    let has_filters = params.descriere.is_some()
        || params.bilete.is_some()
        || params.paginare.page.is_some()
        || params.paginare.items_per_page.is_some();

    let response: Vec<Response<EventPackets>> = if has_filters {
        build_filtered_event_packets(event_packets, &params, &state.base_url)
    } else {
        event_packets
            .into_iter()
            .map(|e| build_simple_event_packet(e, &state.base_url))
            .collect()
    };

    Ok(Json(response))
}

pub async fn get_event_packet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, EventPacketRepoError> {
    let event_packet = state.event_packet_repo.get_event_packet(id).await?;

    let packet_response = build_simple_event_packet(event_packet, &state.base_url);

    Ok(Json(packet_response))
}

pub async fn update_event_packet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateEventPacket>,
) -> Result<impl IntoResponse, EventPacketRepoError> {
    let event_packet = state
        .event_packet_repo
        .update_event_packet(id, payload)
        .await?;

    let packet_response = build_simple_event_packet(event_packet, &state.base_url);

    Ok(Json(packet_response))
}

pub async fn create_event_packet(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateEventPacket>,
) -> Result<impl IntoResponse, EventPacketRepoError> {
    let event_packet = state.event_packet_repo.create_event_packet(payload).await?;

    let packet_response = build_simple_event_packet(event_packet, &state.base_url);

    Ok((StatusCode::CREATED, Json(packet_response)))
}

pub async fn delete_event_packet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, EventPacketRepoError> {
    state.event_packet_repo.delete_event_packet(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_tickets_for_packet(
    State(state): State<Arc<AppState>>,
    Path(packet_id): Path<i32>,
) -> Result<impl IntoResponse, TicketRepoError> {
    let tickets = state.ticket_repo.list_tickets_for_packet(packet_id).await?;

    let wrapped: Vec<Response<Ticket>> = tickets
        .into_iter()
        .map(|t| build_ticket_over_packet(t, packet_id, &state.base_url))
        .collect();

    Ok(Json(wrapped))
}

pub async fn get_ticket_for_packet(
    State(state): State<Arc<AppState>>,
    Path((packet_id, ticket_cod)): Path<(i32, String)>,
) -> Result<impl IntoResponse, TicketRepoError> {
    let ticket = state
        .ticket_repo
        .get_ticket_for_packet(packet_id, &ticket_cod)
        .await?;

    let ticket_response = build_ticket_over_packet(ticket, packet_id, &state.base_url);

    Ok(Json(ticket_response))
}

pub async fn create_ticket_for_packet(
    State(state): State<Arc<AppState>>,
    Path(packet_id): Path<i32>,
    Json(payload): Json<CreateTicket>,
) -> Result<impl IntoResponse, TicketRepoError> {
    let ticket = state
        .ticket_repo
        .create_ticket_for_packet(packet_id, payload)
        .await?;

    let ticket_response = build_ticket_over_packet(ticket, packet_id, &state.base_url);

    Ok((StatusCode::CREATED, Json(ticket_response)))
}

pub async fn update_ticket_for_packet(
    State(state): State<Arc<AppState>>,
    Path((packet_id, ticket_cod)): Path<(i32, String)>,
    Json(payload): Json<UpdateTicket>,
) -> Result<impl IntoResponse, TicketRepoError> {
    let ticket = state
        .ticket_repo
        .update_ticket_for_packet(packet_id, &ticket_cod, payload)
        .await?;

    let ticket_response = build_ticket_over_packet(ticket, packet_id, &state.base_url);

    Ok(Json(ticket_response))
}

pub async fn delete_ticket_for_packet(
    State(state): State<Arc<AppState>>,
    Path((packet_id, ticket_cod)): Path<(i32, String)>,
) -> Result<impl IntoResponse, TicketRepoError> {
    state
        .ticket_repo
        .delete_ticket_for_packet(packet_id, &ticket_cod)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn event_packet_manager_router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/event-packets",
            post(create_event_packet).get(list_event_packets),
        )
        .route(
            "/event-packets/{id}",
            get(get_event_packet)
                .put(update_event_packet)
                .delete(delete_event_packet),
        )
        .route(
            "/event-packets/{id}/tickets",
            get(list_tickets_for_packet).post(create_ticket_for_packet),
        )
        .route(
            "/event-packets/{id}/tickets/{ticket_cod}",
            get(get_ticket_for_packet)
                .put(update_ticket_for_packet)
                .delete(delete_ticket_for_packet),
        )
}
