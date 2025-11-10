use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::Error;
use validator::ValidationErrors;

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

#[derive(Serialize)]
struct ApiErrorResponse {
    error: String,
    details: Vec<String>,
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
    InternalError(Error),
}

#[derive(Debug)]
pub enum JoinPeRepoError {
    DuplicateEntry,
    InvalidReference,
    InternalError(Error),
}

impl From<String> for ApiError {
    fn from(value: String) -> Self {
        ApiError::BadRequest(value.into())
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
                let mut all_messages = Vec::new();
                for error_list in errors.field_errors().values() {
                    for error in *error_list {
                        if let Some(msg) = error.message.as_ref() {
                            all_messages.push(msg.to_string());
                        }
                    }
                }
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
                let (status, title, detail) = match rejection {
                    JsonRejection::JsonDataError(err) => {
                        let msg = err.to_string();
                        let final_msg = if msg.starts_with("unknown field") {
                            msg.split('`')
                                .nth(1)
                                .map(|field_name| format!("Unknown field `{}`", field_name))
                                .unwrap_or("Invalid data format.".to_string())
                        } else {
                            msg
                        };
                        (
                            StatusCode::UNPROCESSABLE_ENTITY,
                            "Invalid JSON Data".to_string(),
                            vec![final_msg],
                        )
                    }
                    JsonRejection::JsonSyntaxError(err) => (
                        StatusCode::BAD_REQUEST,
                        "Invalid JSON Syntax".to_string(),
                        vec![err.to_string()],
                    ),
                    JsonRejection::MissingJsonContentType(_) => (
                        StatusCode::UNSUPPORTED_MEDIA_TYPE,
                        "Missing Content-Type".to_string(),
                        vec!["Expected 'application/json'.".to_string()],
                    ),
                    _ => (
                        StatusCode::BAD_REQUEST,
                        "Bad JSON Request".to_string(),
                        vec![rejection.to_string()],
                    ),
                };
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
        Error::RowNotFound => JoinPeRepoError::InvalidReference,
        e => JoinPeRepoError::InternalError(e),
    }
}
