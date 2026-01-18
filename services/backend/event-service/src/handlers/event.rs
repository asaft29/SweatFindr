use crate::AppState;
use crate::handlers::ticket;
use crate::middleware::{Authorization, UserClaims};
use crate::models::event::{CreateEvent, Event, EventQuery, PatchEvent, UpdateEvent};
use crate::utils::error::{ApiError, map_authorization_error};
use crate::utils::links::{Response, build_filtered_event, build_simple_event};
use axum::extract::rejection::JsonRejection;
use axum::response::IntoResponse;
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put},
};
use std::sync::Arc;
use validator::Validate;

pub fn event_manager_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/events", post(create_event))
        .route(
            "/events/{id}",
            put(update_event)
                .delete(delete_event)
                .patch(patch_event),
        )
        .route(
            "/events/{id}/tickets",
            get(ticket::list_tickets_for_event)
                .post(ticket::create_ticket_for_event),
        )
        .route(
            "/events/{id}/tickets/{cod}",
            get(ticket::get_ticket_for_event)
                .delete(ticket::delete_ticket_for_event),
        )
}

pub fn public_event_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/events", get(list_events))
        .route("/events/{id}", get(get_event))
}

#[utoipa::path(
    get,
    path = "/api/event-manager/events",
    params(
        ("location" = Option<String>, Query, description = "Filter by location of the event"),
        ("name" = Option<String>, Query, description = "Filter by event name")
    ),
    responses(
        (status = 200, description = "List events (optionally filtered by location or name)", body = [Response<Event>]),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Events",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EventQuery>,
) -> Result<impl IntoResponse, ApiError> {
    params.validate()?;

    let events: Vec<Event> = state.event_repo.list_events(params.clone()).await?;

    let has_filters = params.locatie.is_some()
        || params.nume.is_some()
        || params.paginare.page.is_some()
        || params.paginare.items_per_page.is_some();

    let response: Vec<Response<Event>> = if has_filters {
        build_filtered_event(events, &params, &state.base_url)
    } else {
        events
            .into_iter()
            .map(|event| build_simple_event(event, &state.base_url))
            .collect()
    };

    Ok((StatusCode::OK, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/event-manager/events/{id}",
    params(
        ("id" = i32, Path, description = "ID of the event to retrieve")
    ),
    responses(
        (status = 200, description = "Return an event by ID", body = Response<Event>),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 404, description = "Event not found")
    ),
    tag = "Events",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
    let event = state.event_repo.get_event(id).await?;

    let event_response = build_simple_event(event, &state.base_url);

    Ok(Json(event_response))
}

#[utoipa::path(
    put,
    path = "/api/event-manager/events/{id}",
    params(
        ("id" = i32, Path, description = "ID of the event to update")
    ),
    request_body = UpdateEvent,
    responses(
        (status = 204, description = "Event updated"),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Only event owner or admin can update"),
        (status = 404, description = "Event not found")
    ),
    tag = "Events",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_event(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<i32>,
    payload: Result<Json<UpdateEvent>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    if id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    let existing_event = state.event_repo.get_event(id).await?;
    Authorization::can_modify_resource(&user_claims, &existing_event, None)
        .map_err(map_authorization_error)?;

    let Json(payload) = payload?;
    payload.validate()?;

    state.event_repo.update_event(id, payload).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    patch,
    path = "/api/event-manager/events/{id}",
    params(
        ("id" = i32, Path, description = "ID of the event to update")
    ),
    request_body = PatchEvent,
    responses(
        (status = 200, description = "Event partially updated", body = Response<Event>),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Only event owner or admin can update"),
        (status = 404, description = "Event not found")
    ),
    tag = "Events",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn patch_event(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<i32>,
    payload: Result<Json<PatchEvent>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    if id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    let existing_event = state.event_repo.get_event(id).await?;
    Authorization::can_modify_resource(&user_claims, &existing_event, None)
        .map_err(map_authorization_error)?;

    let Json(payload) = payload?;
    payload.validate()?;

    let event = state.event_repo.patch_event(id, payload).await?;
    let event_response = build_simple_event(event, &state.base_url);

    Ok(Json(event_response))
}

#[utoipa::path(
    post,
    path = "/api/event-manager/events",
    request_body = CreateEvent,
    responses(
        (status = 201, description = "Event created successfully", body = Response<Event>),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Requires owner-event role or admin")
    ),
    tag = "Events",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_event(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    payload: Result<Json<CreateEvent>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    Authorization::require_owner_event_or_admin(&user_claims).map_err(map_authorization_error)?;

    let Json(payload) = payload?;
    payload.validate()?;

    let event = state
        .event_repo
        .create_event(user_claims.user_id, payload)
        .await?;

    let event_response = build_simple_event(event, &state.base_url);

    Ok((StatusCode::CREATED, Json(event_response)))
}

#[utoipa::path(
    delete,
    path = "/api/event-manager/events/{id}",
    params(
        ("id" = i32, Path, description = "ID of the event to delete")
    ),
    responses(
        (status = 204, description = "Event deleted successfully"),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Only event owner or admin can delete"),
        (status = 404, description = "Event not found"),
        (status = 409, description = "Cannot delete event with sold tickets")
    ),
    tag = "Events",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_event(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let existing_event = state.event_repo.get_event(id).await?;
    Authorization::can_modify_resource(&user_claims, &existing_event, None)
        .map_err(map_authorization_error)?;
    if id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    let ticket_count = state.ticket_repo.count_tickets_for_event(id).await?;
    if ticket_count > 0 {
        return Err(ApiError::Conflict(format!(
            "Cannot delete event with {} sold ticket(s). Please cancel the tickets first.",
            ticket_count
        )));
    }

    state
        .join_repo
        .update_packets_before_event_deletion(id)
        .await?;

    state.event_repo.delete_event(id).await?;

    Ok(StatusCode::NO_CONTENT)
}
