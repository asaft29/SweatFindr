use super::types::{EventInfo, ExternalServiceError};
use reqwest;
use tracing::info;

pub async fn get_event(
    event_service_url: &str,
    event_id: i32,
) -> Result<EventInfo, ExternalServiceError> {
    let event_url = format!(
        "{}/api/event-manager/events/{}",
        event_service_url, event_id
    );
    info!("Fetching event details from: {}", event_url);

    let response = reqwest::get(&event_url).await.map_err(|e| {
        ExternalServiceError::HttpError(format!("Failed to connect to event-service: {}", e))
    })?;

    if response.status().as_u16() == 404 {
        return Err(ExternalServiceError::NotFound(format!(
            "Event with ID {} does not exist in event-service",
            event_id
        )));
    }

    if !response.status().is_success() {
        return Err(ExternalServiceError::HttpError(format!(
            "Event-service returned status: {}",
            response.status()
        )));
    }

    let event = response.json::<EventInfo>().await.map_err(|e| {
        ExternalServiceError::DeserializationError(format!("Failed to parse event response: {}", e))
    })?;

    info!("Successfully fetched event: {}", event.nume);
    Ok(event)
}
