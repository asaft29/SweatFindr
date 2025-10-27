use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sqlx::Error;

#[derive(Debug)]
pub enum EventRepoError {
    NotFound,
    InvalidReference,
    DuplicateEntry,
    InternalError(Error),
}

#[derive(Debug)]
pub enum EventPacketRepoError {
    NotFound,
    DuplicateName,
    InvalidEventId,
    InternalError(Error),
}

impl IntoResponse for EventRepoError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            EventRepoError::NotFound => (
                StatusCode::NOT_FOUND,
                json!({ "error": "The requested resource was not found." }),
            ),
            EventRepoError::InvalidReference => (
                StatusCode::BAD_REQUEST,
                json!({ "error": "A provided reference, such as an owner ID, is invalid." }),
            ),
            EventRepoError::DuplicateEntry => (
                StatusCode::CONFLICT,
                json!({ "error": "An event with this name already exists." }),
            ),
            EventRepoError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "An internal server error occurred." }),
            ),
        };

        (status, Json(error_message)).into_response()
    }
}

impl IntoResponse for EventPacketRepoError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            EventPacketRepoError::NotFound => (
                StatusCode::NOT_FOUND,
                json!({ "error": "The requested resource was not found." }),
            ),
            EventPacketRepoError::DuplicateName => (
                StatusCode::CONFLICT,
                json!({ "error": "An event packet with this name already exists." }),
            ),
            EventPacketRepoError::InvalidEventId => (
                StatusCode::BAD_REQUEST,
                json!({ "error": "A provided event ID is invalid." }),
            ),
            EventPacketRepoError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "An internal server error occurred." }),
            ),
        };

        (status, Json(error_message)).into_response()
    }
}

pub fn map_sqlx_event_error(err: Error) -> EventRepoError {
    if let Some(db_err) = err.as_database_error() {
        if let Some(code) = db_err.code() {
            match code.as_ref() {
                "23503" => return EventRepoError::InvalidReference,
                "23505" => return EventRepoError::DuplicateEntry,
                _ => {}
            }
        }
    }
    match err {
        Error::RowNotFound => EventRepoError::NotFound,
        e => EventRepoError::InternalError(e),
    }
}

pub fn map_sqlx_packet_error(err: Error) -> EventPacketRepoError {
    if let Some(db_err) = err.as_database_error() {
        if let Some(code) = db_err.code() {
            match code.as_ref() {
                "23503" => return EventPacketRepoError::InvalidEventId,
                "23505" => return EventPacketRepoError::DuplicateName,
                _ => {}
            }
        }
    }
    match err {
        Error::RowNotFound => EventPacketRepoError::NotFound,
        e => EventPacketRepoError::InternalError(e),
    }
}
