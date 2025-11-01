use crate::AppState;
use crate::handlers::ticket;
use crate::models::event_packets::{
    CreateEventPacket, EventPacketQuery, EventPackets, UpdateEventPacket,
};
use crate::shared::error::EventPacketRepoError;
use crate::shared::links::{Response, build_filtered_event_packets, build_simple_event_packet};
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/event-manager/event-packets",
    params(
        ("type" = Option<String>, Query, description = "Filter event packets by description/type"),
        ("available_tickets" = Option<i32>, Query, description = "Filter event packets by available tickets"),
        ("page" = Option<i64>, Query, description = "Pagination page number"),
        ("items_per_page" = Option<i64>, Query, description = "Items per page for pagination")
    ),
    responses(
        (status = 200, description = "List all event packets (optionally filtered)", body = [Response<EventPackets>]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Event Packets"
)]
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

#[utoipa::path(
    get,
    path = "/api/event-manager/event-packets/{id}",
    params(("id" = i32, Path, description = "Event packet ID")),
    responses(
        (status = 200, description = "Get event packet by ID", body = Response<EventPackets>),
        (status = 404, description = "Event packet not found")
    ),
    tag = "Event Packets"
)]
pub async fn get_event_packet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, EventPacketRepoError> {
    let event_packet = state.event_packet_repo.get_event_packet(id).await?;

    let packet_response = build_simple_event_packet(event_packet, &state.base_url);

    Ok(Json(packet_response))
}

#[utoipa::path(
    put,
    path = "/api/event-manager/event-packets/{id}",
    params(("id" = i32, Path, description = "Event packet ID")),
    request_body = UpdateEventPacket,
    responses(
        (status = 200, description = "Update an existing event packet", body = Response<EventPackets>),
        (status = 404, description = "Event packet not found")
    ),
    tag = "Event Packets"
)]
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

#[utoipa::path(
    post,
    path = "/api/event-manager/event-packets",
    request_body = CreateEventPacket,
    responses(
        (status = 201, description = "Create a new event packet", body = Response<EventPackets>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Event Packets"
)]
pub async fn create_event_packet(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateEventPacket>,
) -> Result<impl IntoResponse, EventPacketRepoError> {
    let event_packet = state.event_packet_repo.create_event_packet(payload).await?;

    let packet_response = build_simple_event_packet(event_packet, &state.base_url);

    Ok((StatusCode::CREATED, Json(packet_response)))
}

#[utoipa::path(
    delete,
    path = "/api/event-manager/event-packets/{id}",
    params(("id" = i32, Path, description = "Event packet ID")),
    responses(
        (status = 204, description = "Event packet deleted successfully"),
        (status = 404, description = "Event packet not found")
    ),
    tag = "Event Packets"
)]
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
            get(ticket::list_tickets_for_packet).post(ticket::create_ticket_for_packet),
        )
        .route(
            "/event-packets/{id}/tickets/{ticket_cod}",
            get(ticket::get_ticket_for_packet)
                .put(ticket::update_ticket_for_packet)
                .delete(ticket::delete_ticket_for_packet),
        )
}
