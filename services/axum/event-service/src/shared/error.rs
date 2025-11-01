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

#[derive(Debug)]
pub enum TicketRepoError {
    NotFound,
    DuplicateEntry,
    InvalidReference,
    InternalError(Error),
}

#[derive(Debug)]
pub enum JoinPeRepoError {
    DuplicateEntry,
    InvalidReference,
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

impl IntoResponse for TicketRepoError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            TicketRepoError::NotFound => (
                StatusCode::NOT_FOUND,
                json!({ "error": "The requested ticket was not found." }),
            ),
            TicketRepoError::DuplicateEntry => (
                StatusCode::CONFLICT,
                json!({ "error": "A ticket with this code already exists." }),
            ),
            TicketRepoError::InvalidReference => (
                StatusCode::BAD_REQUEST,
                json!({ "error": "Invalid packet or event ID provided." }),
            ),

            TicketRepoError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "An internal server error occurred." }),
            ),
        };
        (status, Json(error_message)).into_response()
    }
}

impl IntoResponse for JoinPeRepoError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            JoinPeRepoError::DuplicateEntry => (
                StatusCode::CONFLICT,
                json!({ "error": "This event is already in this packet." }),
            ),
            JoinPeRepoError::InvalidReference => (
                StatusCode::BAD_REQUEST,
                json!({ "error": "Invalid packet or event ID provided." }),
            ),
            JoinPeRepoError::InternalError(_) => (
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

pub fn map_sqlx_ticket_error(err: Error) -> TicketRepoError {
    if let Some(db_err) = err.as_database_error() {
        if let Some(code) = db_err.code() {
            match code.as_ref() {
                "23503" => return TicketRepoError::InvalidReference,
                "23505" => return TicketRepoError::DuplicateEntry,
                _ => {}
            }
        }
    }
    match err {
        Error::RowNotFound => TicketRepoError::NotFound,
        e => TicketRepoError::InternalError(e),
    }
}

pub fn map_sqlx_join_pe_error(err: Error) -> JoinPeRepoError {
    if let Some(db_err) = err.as_database_error() {
        if let Some(code) = db_err.code() {
            match code.as_ref() {
                "23503" => return JoinPeRepoError::InvalidReference,
                "23505" => return JoinPeRepoError::DuplicateEntry,
                _ => {}
            }
        }
    }
    JoinPeRepoError::InternalError(err)
}
