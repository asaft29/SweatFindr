use crate::AppState;
use crate::models::event::Event;
use crate::models::event_packets::EventPackets;
use crate::utils::error::{ApiError, map_authorization_error};
use crate::utils::links::{Response, build_event_over_packet, build_packet_over_event};
use crate::middleware::{Authorization, UserClaims};
use axum::Router;
use axum::response::IntoResponse;
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
};
use std::sync::Arc;

pub fn join_pe_manager_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/events/{id}/event-packets", get(list_packets_for_event))
        .route("/event-packets/{id}/events", get(list_events_for_packet))
        .route(
            "/event-packets/{packet_id}/events/{event_id}",
            delete(remove_event_from_packet).post(add_event_to_packet),
        )
}

#[utoipa::path(
    get,
    path = "/api/event-manager/events/{id}/event-packets",
    params(
        ("id" = i32, Path, description = "ID of the event")
    ),
    responses(
        (status = 200, description = "List packets linked to the specified event", body = [Response<EventPackets>]),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "JoinPE",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_packets_for_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if id < 0 {
        return Err(ApiError::BadRequest("Cannot get with negative ID".into()));
    }
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
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 404, description = "Event packet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "JoinPE",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_events_for_packet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
    let events = state.join_repo.get_events_for_packet(id).await?;

    let wrapped: Vec<Response<Event>> = events
        .into_iter()
        .map(|e| build_event_over_packet(e, id, &state.base_url))
        .collect();

    Ok(Json(wrapped))
}

#[utoipa::path(
    post,
    path = "/api/event-manager/event-packets/{packet_id}/events/{event_id}",
    params(
        ("packet_id" = i32, Path, description = "ID of the event packet"),
        ("event_id" = i32, Path, description = "ID of the event to add to the packet")
    ),
    responses(
        (status = 201, description = "Event successfully linked to event packet"),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Must own both event and packet, or be admin"),
        (status = 404, description = "Event or packet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "JoinPE",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn add_event_to_packet(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path((packet_id, event_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, ApiError> {
    if packet_id < 0 {
        return Err(ApiError::BadRequest("Packet ID cannot be negative".into()));
    }
    if event_id < 0 {
        return Err(ApiError::BadRequest("Event ID cannot be negative".into()));
    }

    let event = state.event_repo.get_event(event_id).await?;
    let packet = state.event_packet_repo.get_event_packet(packet_id).await?;

    Authorization::can_modify_resource(&user_claims, &event, None)
        .map_err(map_authorization_error)?;
    Authorization::can_modify_resource(&user_claims, &packet, None)
        .map_err(map_authorization_error)?;

    let _ = state
        .join_repo
        .add_event_to_packet(packet_id, event_id)
        .await?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    delete,
    path = "/api/event-manager/event-packets/{packet_id}/events/{event_id}",
    params(
        ("packet_id" = i32, Path, description = "ID of the event packet"),
        ("event_id" = i32, Path, description = "ID of the event to remove from the packet")
    ),
    responses(
        (status = 204, description = "Event successfully removed from packet, packet capacity updated"),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Must own both event and packet, or be admin"),
        (status = 404, description = "Relationship not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "JoinPE",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn remove_event_from_packet(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path((packet_id, event_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, ApiError> {
    if packet_id < 0 {
        return Err(ApiError::BadRequest("Packet ID cannot be negative".into()));
    }
    if event_id < 0 {
        return Err(ApiError::BadRequest("Event ID cannot be negative".into()));
    }

    let event = state.event_repo.get_event(event_id).await?;
    let packet = state.event_packet_repo.get_event_packet(packet_id).await?;

    Authorization::can_modify_resource(&user_claims, &event, None)
        .map_err(map_authorization_error)?;
    Authorization::can_modify_resource(&user_claims, &packet, None)
        .map_err(map_authorization_error)?;

    state
        .join_repo
        .remove_event_from_packet(packet_id, event_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
