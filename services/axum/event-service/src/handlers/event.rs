use crate::AppState;
use crate::error::EventRepoError;
use crate::handlers::ticket;
use crate::links::EventResponse;
use crate::models::event::{CreateEvent, Event, EventQuery, UpdateEvent};
use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;

pub async fn list_events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EventQuery>,
) -> Result<impl IntoResponse, EventRepoError> {
    let events: Vec<Event> = state.event_repo.list_events(params).await?;

    let event_responses: Vec<EventResponse> = events
        .into_iter()
        .map(|event| EventResponse::new(event, &state.base_url))
        .collect();

    Ok((StatusCode::OK, Json(event_responses)))
}

pub async fn get_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, EventRepoError> {
    let event = state.event_repo.get_event(id).await?;

    let event_response = EventResponse::new(event, &state.base_url);
    Ok(Json(event_response))
}

pub async fn update_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateEvent>,
) -> Result<impl IntoResponse, EventRepoError> {
    let event = state.event_repo.update_event(id, payload).await?;

    let event_response = EventResponse::new(event, &state.base_url);

    Ok(Json(event_response))
}

pub async fn create_event(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateEvent>,
) -> Result<impl IntoResponse, EventRepoError> {
    let event = state.event_repo.create_event(payload).await?;

    let event_response = EventResponse::new(event, &state.base_url);

    Ok((StatusCode::CREATED, Json(event_response)))
}

pub async fn delete_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, EventRepoError> {
    state.event_repo.delete_event(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn event_manager_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/events", get(list_events))
        .route("/events/{id}", get(get_event))
        .route(
            "/events/{id}/tickets",
            get(ticket::get_tickets_for_event).post(ticket::create_ticket_for_event),
        )
        .route(
            "/events/{id}/tickets/{cod}",
            get(ticket::get_ticket_for_event)
                .put(ticket::update_ticket_for_event)
                .delete(ticket::delete_ticket_for_event),
        )
}
