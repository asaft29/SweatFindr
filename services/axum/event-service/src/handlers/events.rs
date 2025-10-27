use crate::AppState;
use crate::error::EventRepoError;
use crate::links::EventResponse;
use crate::models::event::{CreateEvent, UpdateEvent};
use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;

pub async fn get_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<EventResponse>, EventRepoError> {
    let event = state.event_repo.get_event(id).await?;

    let event_response = EventResponse::new(event, &state.base_url);
    Ok(Json(event_response))
}

pub async fn update_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateEvent>,
) -> Result<Json<EventResponse>, EventRepoError> {
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
    Router::new().route("/events", post(create_event)).route(
        "/events/{id}",
        get(get_event).put(update_event).delete(delete_event),
    )
}
