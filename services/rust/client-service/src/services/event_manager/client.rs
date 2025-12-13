use super::types::ExternalServiceError;
use reqwest::{Client, Response, StatusCode};

pub struct EventManagerClient {
    base_url: String,
    http_client: Client,
}

impl EventManagerClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            http_client: Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub async fn get(&self, path: &str) -> Result<Response, ExternalServiceError> {
        self.http_client
            .get(self.url(path))
            .send()
            .await
            .map_err(|e| {
                ExternalServiceError::HttpError(format!(
                    "Failed to connect to event-service: {}",
                    e
                ))
            })
    }

    pub async fn post_with_auth(
        &self,
        path: &str,
        token: &str,
    ) -> Result<Response, ExternalServiceError> {
        self.http_client
            .post(self.url(path))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| {
                ExternalServiceError::HttpError(format!(
                    "Failed to connect to event-service: {}",
                    e
                ))
            })
    }

    pub async fn put_with_auth<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
        token: &str,
    ) -> Result<Response, ExternalServiceError> {
        self.http_client
            .put(self.url(path))
            .header("Authorization", format!("Bearer {}", token))
            .json(body)
            .send()
            .await
            .map_err(|e| {
                ExternalServiceError::HttpError(format!(
                    "Failed to connect to event-service: {}",
                    e
                ))
            })
    }

    pub async fn delete_with_auth(
        &self,
        path: &str,
        token: &str,
    ) -> Result<Response, ExternalServiceError> {
        self.http_client
            .delete(self.url(path))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| {
                ExternalServiceError::HttpError(format!(
                    "Failed to connect to event-service: {}",
                    e
                ))
            })
    }

    pub fn check_status(
        &self,
        response: &Response,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<(), ExternalServiceError> {
        let status = response.status();

        match status {
            StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(()),
            StatusCode::BAD_REQUEST => Err(ExternalServiceError::InvalidReference(format!(
                "Invalid {} ID or request: {}",
                resource_type, resource_id
            ))),
            StatusCode::UNAUTHORIZED => Err(ExternalServiceError::Unauthorized(
                "Unauthorized - Invalid or missing authentication token".to_string(),
            )),
            StatusCode::FORBIDDEN => Err(ExternalServiceError::Forbidden(
                "Forbidden - You don't have permission to access this resource".to_string(),
            )),
            StatusCode::NOT_FOUND => Err(ExternalServiceError::NotFound(format!(
                "{} with ID '{}' does not exist in event-service",
                resource_type, resource_id
            ))),
            StatusCode::CONFLICT => Err(ExternalServiceError::NoSeatsAvailable(format!(
                "No seats available for {} {}",
                resource_type, resource_id
            ))),
            StatusCode::INTERNAL_SERVER_ERROR => Err(ExternalServiceError::HttpError(
                "Event-service internal error".to_string(),
            )),
            StatusCode::SERVICE_UNAVAILABLE => Err(ExternalServiceError::HttpError(
                "Event-service is temporarily unavailable".to_string(),
            )),
            StatusCode::GATEWAY_TIMEOUT => Err(ExternalServiceError::HttpError(
                "Event-service request timeout".to_string(),
            )),
            _ => Err(ExternalServiceError::HttpError(format!(
                "Event-service returned unexpected status: {}",
                status
            ))),
        }
    }
}
