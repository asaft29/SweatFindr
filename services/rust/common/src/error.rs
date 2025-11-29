use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use validator::{ValidationErrors, ValidationErrorsKind};

#[derive(Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub details: Vec<String>,
}

pub fn flatten_validation_errors(errors: &ValidationErrors) -> Vec<String> {
    let mut messages = Vec::new();

    for kind in errors.errors().values() {
        match kind {
            ValidationErrorsKind::Struct(nested_errors) => {
                messages.extend(flatten_validation_errors(nested_errors));
            }
            ValidationErrorsKind::List(list_errors) => {
                for nested_errors in list_errors.values() {
                    messages.extend(flatten_validation_errors(nested_errors));
                }
            }
            ValidationErrorsKind::Field(field_errors) => {
                for error in field_errors {
                    if let Some(message) = &error.message {
                        messages.push(message.to_string());
                    }
                }
            }
        }
    }

    messages
}

pub fn handle_json_rejection(rejection: JsonRejection) -> (StatusCode, String, Vec<String>) {
    match rejection {
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
    }
}

pub fn error_response(status: StatusCode, error: String, details: Vec<String>) -> Response {
    (status, Json(ApiErrorResponse { error, details })).into_response()
}
