use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::Error;
use validator::ValidationErrors;

pub use common::error::{ApiErrorResponse, flatten_validation_errors, handle_json_rejection};

#[derive(Debug)]
pub enum ApiError {
    Validation(ValidationErrors),
    Event(EventRepoError),
    BadRequest(String),
    Json(JsonRejection),
    Packet(EventPacketRepoError),
    Ticket(TicketRepoError),
    Join(JoinPeRepoError),
}

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
    ConstraintViolation,
    NoSeatsAvailable,
    InternalError(Error),
}

#[derive(Debug)]
pub enum JoinPeRepoError {
    DuplicateEntry,
    InvalidReference,
    InvalidPacket,
    InvalidEvent,
    NotFound,
    InternalError(Error),
}

impl From<String> for ApiError {
    fn from(value: String) -> Self {
        ApiError::BadRequest(value)
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        ApiError::Json(rejection)
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        ApiError::Validation(errors)
    }
}

impl From<EventRepoError> for ApiError {
    fn from(error: EventRepoError) -> Self {
        ApiError::Event(error)
    }
}

impl From<EventPacketRepoError> for ApiError {
    fn from(error: EventPacketRepoError) -> Self {
        ApiError::Packet(error)
    }
}

impl From<TicketRepoError> for ApiError {
    fn from(error: TicketRepoError) -> Self {
        ApiError::Ticket(error)
    }
}

impl From<JoinPeRepoError> for ApiError {
    fn from(error: JoinPeRepoError) -> Self {
        ApiError::Join(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::Validation(errors) => {
                let all_messages = flatten_validation_errors(&errors);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ApiErrorResponse {
                        error: "Validation Failed".to_string(),
                        details: all_messages,
                    },
                )
            }

            ApiError::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                ApiErrorResponse {
                    error: "Bad Request".to_string(),
                    details: vec![message],
                },
            ),

            ApiError::Json(rejection) => {
                let (status, title, detail) = handle_json_rejection(rejection);
                (status, ApiErrorResponse { error: title, details: detail })
            }

            ApiError::Event(e) => match e {
                EventRepoError::NotFound => (
                    StatusCode::NOT_FOUND,
                    ApiErrorResponse {
                        error: "Resource Not Found".to_string(),
                        details: vec!["The requested event was not found.".to_string()],
                    },
                ),
                EventRepoError::InvalidReference => (
                    StatusCode::BAD_REQUEST,
                    ApiErrorResponse {
                        error: "Invalid Reference".to_string(),
                        details: vec!["A provided reference, such as an owner ID, is invalid."
                            .to_string()],
                    },
                ),
                EventRepoError::DuplicateEntry => (
                    StatusCode::CONFLICT,
                    ApiErrorResponse {
                        error: "Duplicate Entry".to_string(),
                        details: vec!["An event with this name already exists.".to_string()],
                    },
                ),
                EventRepoError::InternalError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiErrorResponse {
                        error: "Internal Server Error".to_string(),
                        details: vec!["An internal server error occurred.".to_string()],
                    },
                ),
            },

            ApiError::Packet(e) => match e {
                EventPacketRepoError::NotFound => (
                    StatusCode::NOT_FOUND,
                    ApiErrorResponse {
                        error: "Resource Not Found".to_string(),
                        details: vec!["The requested event packet was not found.".to_string()],
                    },
                ),
                EventPacketRepoError::DuplicateName => (
                    StatusCode::CONFLICT,
                    ApiErrorResponse {
                        error: "Duplicate Entry".to_string(),
                        details: vec!["An event packet with this name already exists."
                            .to_string()],
                    },
                ),
                EventPacketRepoError::InvalidEventId => (
                    StatusCode::BAD_REQUEST,
                    ApiErrorResponse {
                        error: "Invalid Reference".to_string(),
                        details: vec!["A provided event ID is invalid.".to_string()],
                    },
                ),
                EventPacketRepoError::InternalError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiErrorResponse {
                        error: "Internal Server Error".to_string(),
                        details: vec!["An internal server error occurred.".to_string()],
                    },
                ),
            },

            ApiError::Ticket(e) => match e {
                TicketRepoError::NotFound => (
                    StatusCode::NOT_FOUND,
                    ApiErrorResponse {
                        error: "Resource Not Found".to_string(),
                        details: vec!["The requested ticket was not found.".to_string()],
                    },
                ),
                TicketRepoError::DuplicateEntry => (
                    StatusCode::CONFLICT,
                    ApiErrorResponse {
                        error: "Duplicate Entry".to_string(),
                        details: vec!["A ticket with this code already exists.".to_string()],
                    },
                ),
                TicketRepoError::InvalidReference => (
                    StatusCode::BAD_REQUEST,
                    ApiErrorResponse {
                        error: "Invalid Reference".to_string(),
                        details: vec!["Invalid packet or event ID provided.".to_string()],
                    },
                ),
                TicketRepoError::ConstraintViolation => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ApiErrorResponse {
                        error: "Constraint Violation".to_string(),
                        details: vec![
                            "A ticket must belong to EITHER a packet OR an event, not both or neither."
                                .to_string(),
                        ],
                    },
                ),
                TicketRepoError::NoSeatsAvailable => (
                    StatusCode::CONFLICT,
                    ApiErrorResponse {
                        error: "No Seats Available".to_string(),
                        details: vec!["No seats available for this event or packet.".to_string()],
                    },
                ),
                TicketRepoError::InternalError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiErrorResponse {
                        error: "Internal Server Error".to_string(),
                        details: vec!["An internal server error occurred.".to_string()],
                    },
                ),
            },

            ApiError::Join(e) => match e {
                JoinPeRepoError::DuplicateEntry => (
                    StatusCode::CONFLICT,
                    ApiErrorResponse {
                        error: "Duplicate Entry".to_string(),
                        details: vec!["This event is already in this packet.".to_string()],
                    },
                ),
                JoinPeRepoError::InvalidReference => (
                    StatusCode::BAD_REQUEST,
                    ApiErrorResponse {
                        error: "Invalid Reference".to_string(),
                        details: vec!["Invalid packet or event ID provided.".to_string()],
                    },
                ),
                JoinPeRepoError::InvalidPacket => (
                    StatusCode::NOT_FOUND,
                    ApiErrorResponse {
                        error: "Invalid Packet ID".to_string(),
                        details: vec!["The specified packet does not exist.".to_string()],
                    },
                ),
                JoinPeRepoError::InvalidEvent => (
                    StatusCode::NOT_FOUND,
                    ApiErrorResponse {
                        error: "Invalid Event ID".to_string(),
                        details: vec!["The specified event does not exist.".to_string()],
                    },
                ),
                JoinPeRepoError::NotFound => (
                    StatusCode::NOT_FOUND,
                    ApiErrorResponse {
                        error: "Resource Not Found".to_string(),
                        details: vec!["The event-packet relationship was not found.".to_string()],
                    },
                ),
                JoinPeRepoError::InternalError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiErrorResponse {
                        error: "Internal Server Error".to_string(),
                        details: vec!["An internal server error occurred.".to_string()],
                    },
                ),
            },
        };

        (status, Json(body)).into_response()
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
                "23514" => return TicketRepoError::ConstraintViolation,
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
    match err {
        Error::RowNotFound => JoinPeRepoError::NotFound,
        e => JoinPeRepoError::InternalError(e),
    }
}
