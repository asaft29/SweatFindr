use super::client::EventManagerClient;
use super::types::{ExternalServiceError, PacketInfo};
use tracing::info;

pub async fn get_packet(
    client: &EventManagerClient,
    packet_id: i32,
) -> Result<PacketInfo, ExternalServiceError> {
    let path = format!("/api/event-manager/event-packets/{}", packet_id);
    info!("Fetching packet details for ID: {}", packet_id);

    let response = client.get(&path).await?;
    client.check_status(&response, "Packet", &packet_id.to_string())?;

    let packet = response.json::<PacketInfo>().await.map_err(|e| {
        ExternalServiceError::DeserializationError(format!(
            "Failed to parse packet response: {}",
            e
        ))
    })?;

    info!("Successfully fetched packet: {}", packet.nume);
    Ok(packet)
}
