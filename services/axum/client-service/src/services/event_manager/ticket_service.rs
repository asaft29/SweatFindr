use super::event_service_client;
use super::packet_service;
use super::types::{ExternalServiceError, HateoasRequest, TicketDetails, TicketInfo};
use reqwest;
use tracing::info;

pub async fn validate_ticket(
    event_service_url: &str,
    ticket_cod: &str,
) -> Result<(), ExternalServiceError> {
    let ticket_url = format!(
        "{}/api/event-manager/tickets/{}",
        event_service_url, ticket_cod
    );

    let response = reqwest::get(&ticket_url).await.map_err(|e| {
        ExternalServiceError::HttpError(format!("Failed to connect to event-service: {}", e))
    })?;

    if response.status().as_u16() == 404 {
        return Err(ExternalServiceError::NotFound(format!(
            "Ticket with code '{}' does not exist in event-service",
            ticket_cod
        )));
    }

    if !response.status().is_success() {
        return Err(ExternalServiceError::HttpError(format!(
            "Event-service returned status: {}",
            response.status()
        )));
    }

    Ok(())
}

pub async fn get_ticket_details(
    event_service_url: &str,
    ticket_cod: &str,
) -> Result<TicketDetails, ExternalServiceError> {
    let ticket_url = format!(
        "{}/api/event-manager/tickets/{}",
        event_service_url, ticket_cod
    );

    info!("Fetching ticket details from: {}", ticket_url);

    let response = reqwest::get(&ticket_url).await.map_err(|e| {
        ExternalServiceError::HttpError(format!("Failed to connect to event-service: {}", e))
    })?;

    if response.status().as_u16() == 404 {
        return Err(ExternalServiceError::NotFound(format!(
            "Ticket with code '{}' does not exist in event-service",
            ticket_cod
        )));
    }

    if !response.status().is_success() {
        return Err(ExternalServiceError::HttpError(format!(
            "Event-service returned status: {}",
            response.status()
        )));
    }

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
        event_info = event_service_client::get_event(event_service_url, event_id)
            .await
            .ok();
    }

    if let Some(packet_id) = ticket_info.pachetid {
        packet_info = packet_service::get_packet(event_service_url, packet_id)
            .await
            .ok();
    }

    Ok(TicketDetails {
        ticket: ticket_info,
        event: event_info,
        packet: packet_info,
    })
}

pub async fn create_ticket_for_event(
    event_service_url: &str,
    event_id: i32,
) -> Result<TicketDetails, ExternalServiceError> {
    let create_url = format!(
        "{}/api/event-manager/events/{}/tickets",
        event_service_url, event_id
    );

    info!("Creating ticket for event {} at: {}", event_id, create_url);

    let response = reqwest::Client::new()
        .post(&create_url)
        .send()
        .await
        .map_err(|e| {
            ExternalServiceError::HttpError(format!("Failed to connect to event-service: {}", e))
        })?;

    let status = response.status();

    if status.as_u16() == 404 {
        return Err(ExternalServiceError::InvalidReference(format!(
            "Event with ID {} does not exist in event-service",
            event_id
        )));
    }

    if status.as_u16() == 400 {
        return Err(ExternalServiceError::InvalidReference(format!(
            "Invalid event ID: {}",
            event_id
        )));
    }

    if status.as_u16() == 409 {
        return Err(ExternalServiceError::NoSeatsAvailable(format!(
            "No seats available for event {}",
            event_id
        )));
    }

    if !status.is_success() {
        return Err(ExternalServiceError::HttpError(format!(
            "Event-service returned status: {}",
            status
        )));
    }

    let response_data: HateoasRequest<TicketInfo> = response.json().await.map_err(|e| {
        ExternalServiceError::DeserializationError(format!(
            "Failed to parse ticket response: {}",
            e
        ))
    })?;
    let event_info = event_service_client::get_event(event_service_url, event_id)
        .await
        .ok();

    Ok(TicketDetails {
        ticket: response_data.data,
        event: event_info,
        packet: None,
    })
}

pub async fn create_ticket_for_packet(
    event_service_url: &str,
    packet_id: i32,
) -> Result<TicketDetails, ExternalServiceError> {
    let create_url = format!(
        "{}/api/event-manager/event-packets/{}/tickets",
        event_service_url, packet_id
    );

    info!(
        "Creating ticket for packet {} at: {}",
        packet_id, create_url
    );

    let response = reqwest::Client::new()
        .post(&create_url)
        .send()
        .await
        .map_err(|e| {
            ExternalServiceError::HttpError(format!("Failed to connect to event-service: {}", e))
        })?;

    let status = response.status();

    if status.as_u16() == 404 {
        return Err(ExternalServiceError::InvalidReference(format!(
            "Packet with ID {} does not exist in event-service",
            packet_id
        )));
    }

    if status.as_u16() == 400 {
        return Err(ExternalServiceError::InvalidReference(format!(
            "Invalid packet ID: {}",
            packet_id
        )));
    }

    if status.as_u16() == 409 {
        return Err(ExternalServiceError::NoSeatsAvailable(format!(
            "No seats available for packet {}",
            packet_id
        )));
    }

    if !status.is_success() {
        return Err(ExternalServiceError::HttpError(format!(
            "Event-service returned status: {}",
            status
        )));
    }

    let response_data: HateoasRequest<TicketInfo> = response.json().await.map_err(|e| {
        ExternalServiceError::DeserializationError(format!(
            "Failed to parse ticket response: {}",
            e
        ))
    })?;

    let packet_info = packet_service::get_packet(event_service_url, packet_id)
        .await
        .ok();

    Ok(TicketDetails {
        ticket: response_data.data,
        event: None,
        packet: packet_info,
    })
}

pub async fn delete_ticket(
    event_service_url: &str,
    ticket_cod: &str,
) -> Result<(), ExternalServiceError> {
    let delete_url = format!(
        "{}/api/event-manager/tickets/{}",
        event_service_url, ticket_cod
    );

    info!("Deleting ticket {} from event-service", ticket_cod);

    let response = reqwest::Client::new()
        .delete(&delete_url)
        .send()
        .await
        .map_err(|e| {
            ExternalServiceError::HttpError(format!("Failed to connect to event-service: {}", e))
        })?;

    let status = response.status();

    if status.as_u16() == 404 {
        return Err(ExternalServiceError::NotFound(format!(
            "Ticket with code '{}' does not exist in event-service",
            ticket_cod
        )));
    }

    if !status.is_success() {
        return Err(ExternalServiceError::HttpError(format!(
            "Event-service returned status: {}",
            status
        )));
    }

    Ok(())
}
