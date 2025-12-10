use super::client::EventManagerClient;
use super::types::{EventInfo, ExternalServiceError};
use tracing::info;

pub async fn get_event(
    client: &EventManagerClient,
    event_id: i32,
) -> Result<EventInfo, ExternalServiceError> {
    let path = format!("/api/event-manager/events/{}", event_id);
    info!("Fetching event details for ID: {}", event_id);

    let response = client.get(&path).await?;
    client.check_status(&response, "Event", &event_id.to_string())?;

    let event = response.json::<EventInfo>().await.map_err(|e| {
        ExternalServiceError::DeserializationError(format!("Failed to parse event response: {}", e))
    })?;

    info!("Successfully fetched event: {}", event.nume);
    Ok(event)
}
