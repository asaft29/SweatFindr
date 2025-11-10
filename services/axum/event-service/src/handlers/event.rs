use crate::AppState;
use crate::handlers::ticket;
use crate::models::event::{CreateEvent, Event, EventQuery, UpdateEvent};
use crate::shared::error::ApiError;
use crate::shared::links::{Response, build_filtered_event, build_simple_event};
use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use std::sync::Arc;
use validator::Validate;

#[utoipa::path(
    get,
    path = "/api/event-manager/events",
    params(
        ("location" = Option<String>, Query, description = "Filter by location of the event"),
        ("name" = Option<String>, Query, description = "Filter by event name")
    ),
    responses(
        (status = 200, description = "List events (optionally filtered by location or name)", body = [Response<Event>]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Events"
)]
pub async fn list_events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EventQuery>,
) -> Result<impl IntoResponse, ApiError> {
    params.validate()?;

    let events: Vec<Event> = state.event_repo.list_events(params.clone()).await?;

    let has_filters = params.locatie.is_some() || params.nume.is_some();

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
        (status = 404, description = "Event not found")
    ),
    tag = "Events"
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
        (status = 200, description = "Updated event", body = Response<Event>),
        (status = 404, description = "Event not found")
    ),
    tag = "Events"
)]
pub async fn update_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateEvent>,
) -> Result<impl IntoResponse, ApiError> {
    if id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
    payload.validate()?;

    let event = state.event_repo.update_event(id, payload).await?;

    let event_response = build_simple_event(event, &state.base_url);

    Ok(Json(event_response))
}

#[utoipa::path(
    post,
    path = "/api/event-manager/events",
    request_body = CreateEvent,
    responses(
        (status = 201, description = "Event created successfully", body = Response<Event>)
    ),
    tag = "Events"
)]
pub async fn create_event(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateEvent>,
) -> Result<impl IntoResponse, ApiError> {
    payload.validate()?;

    let event = state.event_repo.create_event(payload).await?;

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
        (status = 404, description = "Event not found")
    ),
    tag = "Events"
)]
pub async fn delete_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
    state.event_repo.delete_event(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub fn event_manager_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/events", get(list_events).post(create_event))
        .route(
            "/events/{id}",
            get(get_event).put(update_event).delete(delete_event),
        )
        .route(
            "/events/{id}/tickets",
            get(ticket::list_tickets_for_event).post(ticket::create_ticket_for_event),
        )
        .route(
            "/events/{id}/tickets/{cod}",
            get(ticket::get_ticket_for_event)
                .put(ticket::update_ticket_for_event)
                .delete(ticket::delete_ticket_for_event),
        )
}
