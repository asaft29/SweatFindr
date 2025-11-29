use super::types::{ExternalServiceError, PacketInfo};
use reqwest;
use tracing::info;

pub async fn get_packet(
    event_service_url: &str,
    packet_id: i32,
) -> Result<PacketInfo, ExternalServiceError> {
    let packet_url = format!(
        "{}/api/event-manager/event-packets/{}",
        event_service_url, packet_id
    );
    info!("Fetching packet details from: {}", packet_url);

    let response = reqwest::get(&packet_url).await.map_err(|e| {
        ExternalServiceError::HttpError(format!("Failed to connect to event-service: {}", e))
    })?;

    if response.status().as_u16() == 404 {
        return Err(ExternalServiceError::NotFound(format!(
            "Packet with ID {} does not exist in event-service",
            packet_id
        )));
    }

    if !response.status().is_success() {
        return Err(ExternalServiceError::HttpError(format!(
            "Event-service returned status: {}",
            response.status()
        )));
    }

    let packet = response.json::<PacketInfo>().await.map_err(|e| {
        ExternalServiceError::DeserializationError(format!(
            "Failed to parse packet response: {}",
            e
        ))
    })?;

    info!("Successfully fetched packet: {}", packet.nume);
    Ok(packet)
}
