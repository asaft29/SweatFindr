use crate::models::event::Event;
use crate::repositories::event_repo::{EventRepo, RepoError};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, put},
};
use serde::Serialize;
use std::sync::Arc;

impl Into<StatusCode> for RepoError {
    fn into(self) -> StatusCode {
        match self {
            RepoError::NotFound => StatusCode::NOT_FOUND,
            RepoError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn get_event(
    State(repo): State<Arc<EventRepo>>,
    Path(id): Path<i32>,
) -> Result<Json<Event>, StatusCode> {
    let result = repo.get_event(id).await;

    match result {
        Ok(event) => Ok(Json(event)),
        Err(e) => Err(e.into()),
    }
}

pub fn event_manager_router() -> Router<Arc<EventRepo>> {
    let events_router = Router::new().route("/{id}", get(get_event));

    Router::new().nest("/events", events_router)
}
