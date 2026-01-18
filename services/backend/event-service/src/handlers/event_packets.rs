use crate::AppState;
use crate::handlers::ticket;
use crate::middleware::{Authorization, UserClaims};
use crate::models::event_packets::{
    CreateEventPacket, EventPacketQuery, EventPackets, PatchEventPacket, UpdateEventPacket,
};
use crate::utils::error::{ApiError, map_authorization_error};
use crate::utils::links::{Response, build_filtered_event_packets, build_simple_event_packet};
use axum::extract::Query;
use axum::extract::rejection::JsonRejection;
use axum::response::IntoResponse;
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
};
use std::sync::Arc;
use validator::Validate;

pub fn event_packet_manager_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/event-packets", post(create_event_packet))
        .route(
            "/event-packets/{id}",
            put(update_event_packet)
                .delete(delete_event_packet)
                .patch(patch_event_packet),
        )
        .route(
            "/event-packets/{id}/tickets",
            get(ticket::list_tickets_for_packet)
                .post(ticket::create_ticket_for_packet),
        )
        .route(
            "/event-packets/{id}/tickets/{ticket_cod}",
            get(ticket::get_ticket_for_packet)
                .delete(ticket::delete_ticket_for_packet),
        )
}

pub fn public_event_packet_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/event-packets", get(list_event_packets))
        .route("/event-packets/{id}", get(get_event_packet))
}

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
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Event Packets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_event_packets(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EventPacketQuery>,
) -> Result<impl IntoResponse, ApiError> {
    params.validate()?;

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
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 404, description = "Event packet not found")
    ),
    tag = "Event Packets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_event_packet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
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
        (status = 204, description = "Event packet updated"),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Only packet owner or admin can update"),
        (status = 404, description = "Event packet not found"),
        (status = 409, description = "Cannot modify package with sold tickets")
    ),
    tag = "Event Packets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_event_packet(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<i32>,
    payload: Result<Json<UpdateEventPacket>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    if id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    let existing_packet = state.event_packet_repo.get_event_packet(id).await?;
    Authorization::can_modify_resource(&user_claims, &existing_packet, None)
        .map_err(map_authorization_error)?;

    let ticket_count = state.ticket_repo.count_tickets_for_packet(id).await?;
    if ticket_count > 0 {
        return Err(ApiError::Conflict(format!(
            "Cannot modify package with {} sold ticket(s).",
            ticket_count
        )));
    }

    let Json(payload) = payload?;
    payload.validate()?;

    state
        .event_packet_repo
        .update_event_packet(id, payload)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    patch,
    path = "/api/event-manager/event-packets/{id}",
    params(("id" = i32, Path, description = "Event packet ID")),
    request_body = PatchEventPacket,
    responses(
        (status = 200, description = "Event packet partially updated", body = Response<EventPackets>),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Only packet owner or admin can update"),
        (status = 404, description = "Event packet not found"),
        (status = 409, description = "Cannot modify package with sold tickets")
    ),
    tag = "Event Packets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn patch_event_packet(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<i32>,
    payload: Result<Json<PatchEventPacket>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    if id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    let existing_packet = state.event_packet_repo.get_event_packet(id).await?;
    Authorization::can_modify_resource(&user_claims, &existing_packet, None)
        .map_err(map_authorization_error)?;

    let ticket_count = state.ticket_repo.count_tickets_for_packet(id).await?;
    if ticket_count > 0 {
        return Err(ApiError::Conflict(format!(
            "Cannot modify package with {} sold ticket(s).",
            ticket_count
        )));
    }

    let Json(payload) = payload?;
    payload.validate()?;

    let event_packet = state
        .event_packet_repo
        .patch_event_packet(id, payload)
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
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Requires owner-event role or admin"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Event Packets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_event_packet(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    payload: Result<Json<CreateEventPacket>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    Authorization::require_owner_event_or_admin(&user_claims).map_err(map_authorization_error)?;

    let Json(payload) = payload?;
    payload.validate()?;

    let event_packet = state
        .event_packet_repo
        .create_event_packet(user_claims.user_id, payload)
        .await?;

    let packet_response = build_simple_event_packet(event_packet, &state.base_url);

    Ok((StatusCode::CREATED, Json(packet_response)))
}

#[utoipa::path(
    delete,
    path = "/api/event-manager/event-packets/{id}",
    params(("id" = i32, Path, description = "Event packet ID")),
    responses(
        (status = 204, description = "Event packet deleted successfully"),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Only packet owner or admin can delete"),
        (status = 404, description = "Event packet not found"),
        (status = 409, description = "Cannot delete package with sold tickets")
    ),
    tag = "Event Packets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_event_packet(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    let existing_packet = state.event_packet_repo.get_event_packet(id).await?;
    Authorization::can_modify_resource(&user_claims, &existing_packet, None)
        .map_err(map_authorization_error)?;

    let ticket_count = state.ticket_repo.count_tickets_for_packet(id).await?;
    if ticket_count > 0 {
        return Err(ApiError::Conflict(format!(
            "Cannot delete package with {} sold ticket(s). Please cancel the tickets first.",
            ticket_count
        )));
    }

    state.event_packet_repo.delete_event_packet(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
