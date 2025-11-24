use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use validator::ValidationErrors;

pub use common::error::{ApiErrorResponse, flatten_validation_errors, handle_json_rejection};

#[derive(Debug)]
pub enum ClientApiError {
    Validation(ValidationErrors),
    Client(ClientRepoError),
    BadRequest(String),
    Json(JsonRejection),
    ExternalServiceError(String),
    NotFound(String),
    Conflict(String),
}

#[derive(Debug)]
pub enum ClientRepoError {
    NotFound(String),
    DuplicateEmail(String),
    DatabaseError(String),
    InvalidObjectId(String),
}

impl IntoResponse for ClientApiError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match self {
            ClientApiError::Validation(errors) => {
                let details = flatten_validation_errors(&errors);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Validation Failed".to_string(),
                    Some(details),
                )
            }
            ClientApiError::Client(e) => match e {
                ClientRepoError::NotFound(msg) => (
                    StatusCode::NOT_FOUND,
                    "Client Not Found".to_string(),
                    Some(vec![msg]),
                ),
                ClientRepoError::DuplicateEmail(msg) => (
                    StatusCode::CONFLICT,
                    "Duplicate Email".to_string(),
                    Some(vec![msg]),
                ),
                ClientRepoError::DatabaseError(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database Error".to_string(),
                    Some(vec![msg]),
                ),
                ClientRepoError::InvalidObjectId(msg) => (
                    StatusCode::BAD_REQUEST,
                    "Invalid Object ID".to_string(),
                    Some(vec![msg]),
                ),
            },
            ClientApiError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "Bad Request".to_string(),
                Some(vec![msg]),
            ),
            ClientApiError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "Not Found".to_string(),
                Some(vec![msg]),
            ),
            ClientApiError::Conflict(msg) => (
                StatusCode::CONFLICT,
                "Conflict".to_string(),
                Some(vec![msg]),
            ),
            ClientApiError::ExternalServiceError(msg) => (
                StatusCode::BAD_GATEWAY,
                "External Service Error".to_string(),
                Some(vec![msg]),
            ),
            ClientApiError::Json(rejection) => {
                let (status, title, detail) = handle_json_rejection(rejection);
                (status, title, Some(detail))
            }
        };

        let body = if let Some(details) = details {
            json!({
                "error": error_message,
                "details": details
            })
        } else {
            json!({
                "error": error_message
            })
        };

        (status, Json(body)).into_response()
    }
}

impl From<ValidationErrors> for ClientApiError {
    fn from(errors: ValidationErrors) -> Self {
        ClientApiError::Validation(errors)
    }
}

impl From<ClientRepoError> for ClientApiError {
    fn from(error: ClientRepoError) -> Self {
        ClientApiError::Client(error)
    }
}

impl From<JsonRejection> for ClientApiError {
    fn from(rejection: JsonRejection) -> Self {
        ClientApiError::Json(rejection)
    }
}
