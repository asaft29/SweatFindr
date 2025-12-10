use super::client::EventManagerClient;
use super::event_service_client;
use super::packet_service;
use super::types::{
    BindTicketRequest, ExternalServiceError, HateoasRequest, TicketDetails, TicketInfo,
};
use tracing::info;
use uuid::Uuid;

pub async fn validate_ticket(
    client: &EventManagerClient,
    ticket_cod: &str,
) -> Result<(), ExternalServiceError> {
    let path = format!("/api/event-manager/tickets/{}", ticket_cod);
    let response = client.get(&path).await?;
    client.check_status(&response, "Ticket", ticket_cod)?;
    Ok(())
}

pub async fn get_ticket_details(
    client: &EventManagerClient,
    ticket_cod: &str,
) -> Result<TicketDetails, ExternalServiceError> {
    let path = format!("/api/event-manager/tickets/{}", ticket_cod);

    info!("Fetching ticket details from: {}", path);

    let response = client.get(&path).await?;
    client.check_status(&response, "Ticket", ticket_cod)?;

    let ticket_info: TicketInfo = response.json().await.map_err(|e| {
        ExternalServiceError::DeserializationError(format!(
            "Failed to parse ticket response: {}",
            e
        ))
    })?;

    info!(
        "Ticket info: event_id={:?}, packet_id={:?}",
        ticket_info.evenimentid, ticket_info.pachetid
    );

    let mut event_info = None;
    let mut packet_info = None;

    if let Some(event_id) = ticket_info.evenimentid {
        event_info = event_service_client::get_event(client, event_id).await.ok();
    }

    if let Some(packet_id) = ticket_info.pachetid {
        packet_info = packet_service::get_packet(client, packet_id).await.ok();
    }

    Ok(TicketDetails {
        ticket: ticket_info,
        event: event_info,
        packet: packet_info,
    })
}

pub async fn create_ticket_for_event(
    client: &EventManagerClient,
    event_id: i32,
    service_token: &str,
) -> Result<TicketDetails, ExternalServiceError> {
    let ticket_code = Uuid::now_v7().to_string();

    let path = format!("/api/event-manager/tickets/{}", ticket_code);

    info!(
        "Creating ticket {} for event {} (using service token)",
        ticket_code, event_id
    );

    let payload = BindTicketRequest {
        id_event: Some(event_id),
        id_pachet: None,
    };

    let response = client.put_with_auth(&path, &payload, service_token).await?;
    client.check_status(&response, "Ticket", &ticket_code)?;

    let response_data: HateoasRequest<TicketInfo> = response.json().await.map_err(|e| {
        ExternalServiceError::DeserializationError(format!(
            "Failed to parse ticket response: {}",
            e
        ))
    })?;

    let event_info = event_service_client::get_event(client, event_id).await.ok();

    Ok(TicketDetails {
        ticket: response_data.data,
        event: event_info,
        packet: None,
    })
}

pub async fn create_ticket_for_packet(
    client: &EventManagerClient,
    packet_id: i32,
    service_token: &str,
) -> Result<TicketDetails, ExternalServiceError> {
    let ticket_code = Uuid::now_v7().to_string();

    let path = format!("/api/event-manager/tickets/{}", ticket_code);

    info!(
        "Creating ticket {} for packet {} (using service token)",
        ticket_code, packet_id
    );

    let payload = BindTicketRequest {
        id_event: None,
        id_pachet: Some(packet_id),
    };

    let response = client.put_with_auth(&path, &payload, service_token).await?;
    client.check_status(&response, "Ticket", &ticket_code)?;

    let response_data: HateoasRequest<TicketInfo> = response.json().await.map_err(|e| {
        ExternalServiceError::DeserializationError(format!(
            "Failed to parse ticket response: {}",
            e
        ))
    })?;

    let packet_info = packet_service::get_packet(client, packet_id).await.ok();

    Ok(TicketDetails {
        ticket: response_data.data,
        event: None,
        packet: packet_info,
    })
}

pub async fn delete_ticket(
    client: &EventManagerClient,
    ticket_cod: &str,
    service_token: &str,
) -> Result<(), ExternalServiceError> {
    let path = format!("/api/event-manager/tickets/{}", ticket_cod);

    info!(
        "Deleting ticket {} from event-service (using service token)",
        ticket_cod
    );

    let response = client.delete_with_auth(&path, service_token).await?;
    client.check_status(&response, "Ticket", ticket_cod)?;

    Ok(())
}
