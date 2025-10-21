use crate::AppState;
use crate::links::{EventResponse, EventResponseWrapper};
use crate::models::event::{CreateEvent, UpdateEvent};
use crate::repositories::event_repo::RepoError;
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
) -> Result<Json<EventResponseWrapper>, RepoError> {
    let event = state.repo.get_event(id).await?;

    let event_response = EventResponse::new(event, &state.base_url);
    let final_response = EventResponseWrapper {
        event: event_response,
    };
    Ok(Json(final_response))
}

pub async fn update_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateEvent>,
) -> Result<Json<EventResponseWrapper>, RepoError> {
    let event = state.repo.update_event(id, payload).await?;

    let event_response = EventResponse::new(event, &state.base_url);

    let final_response = EventResponseWrapper {
        event: event_response,
    };
    Ok(Json(final_response))
}

pub async fn create_event(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateEvent>,
) -> Result<impl IntoResponse, RepoError> {
    let event = state.repo.create_event(payload).await?;

    let event_response = EventResponse::new(event, &state.base_url);

    let final_response = EventResponseWrapper {
        event: event_response,
    };
    Ok((StatusCode::CREATED, Json(final_response)))
}

pub async fn delete_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, RepoError> {
    state.repo.delete_event(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn event_manager_router() -> Router<Arc<AppState>> {
    Router::new().route("/events", post(create_event)).route(
        "/events/{id}",
        get(get_event).put(update_event).delete(delete_event),
    )
}
