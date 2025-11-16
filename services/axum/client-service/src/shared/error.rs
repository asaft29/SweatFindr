use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use validator::ValidationErrors;

#[derive(Debug)]
pub enum ApiError {
    Validation(ValidationErrors),
    Client(ClientRepoError),
    BadRequest(String),
    Json(JsonRejection),
    ExternalServiceError(String),
}

#[derive(Debug)]
pub enum ClientRepoError {
    NotFound(String),
    DuplicateEmail(String),
    DatabaseError(String),
    InvalidObjectId(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match self {
            ApiError::Validation(errors) => {
                let details = flatten_validation_errors(&errors);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Validation Failed".to_string(),
                    Some(details),
                )
            }
            ApiError::Client(e) => match e {
                ClientRepoError::NotFound(msg) => {
                    (StatusCode::NOT_FOUND, "Client Not Found".to_string(), Some(vec![msg]))
                }
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
            ApiError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "Bad Request".to_string(), Some(vec![msg]))
            }
            ApiError::ExternalServiceError(msg) => {
                (StatusCode::BAD_GATEWAY, "External Service Error".to_string(), Some(vec![msg]))
            }
            ApiError::Json(rejection) => {
                let (status, title, detail) = match rejection {
                    JsonRejection::JsonDataError(err) => {
                        let msg = err.to_string();
                        let final_msg = if msg.contains("unknown field") {
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

fn flatten_validation_errors(errors: &ValidationErrors) -> Vec<String> {
    let mut messages = Vec::new();

    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            let message = error
                .message
                .as_ref()
                .map(|m| m.to_string())
                .unwrap_or_else(|| format!("Invalid value for field: {}", field));
            messages.push(message);
        }
    }

    messages
}

impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        ApiError::Validation(errors)
    }
}

impl From<ClientRepoError> for ApiError {
    fn from(error: ClientRepoError) -> Self {
        ApiError::Client(error)
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        ApiError::Json(rejection)
    }
}
