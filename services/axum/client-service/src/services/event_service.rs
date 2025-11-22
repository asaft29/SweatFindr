use reqwest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug)]
pub enum EventServiceError {
    TicketNotFound(String),
    HttpError(String),
    DeserializationError(String),
    NoSeatsAvailable(String),
    InvalidReference(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TicketInfo {
    pub cod: String,
    pub pachetid: Option<i32>,
    pub evenimentid: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EventInfo {
    pub id: i32,
    pub id_owner: i32,
    pub nume: String,
    pub locatie: String,
    pub descriere: String,
    pub numarlocuri: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PacketInfo {
    pub id: i32,
    pub id_owner: i32,
    pub nume: String,
    pub locatie: String,
    pub descriere: String,
    pub numarlocuri: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TicketDetails {
    pub ticket: TicketInfo,
    pub event: Option<EventInfo>,
    pub packet: Option<PacketInfo>,
}

pub async fn validate_ticket(
    event_service_url: &str,
    ticket_cod: &str,
) -> Result<(), EventServiceError> {
    let ticket_url = format!(
        "{}/api/event-manager/tickets/{}",
        event_service_url, ticket_cod
    );

    let response = reqwest::get(&ticket_url).await.map_err(|e| {
        EventServiceError::HttpError(format!("Failed to connect to event-service: {}", e))
    })?;

    if response.status().as_u16() == 404 {
        return Err(EventServiceError::TicketNotFound(format!(
            "Ticket with code '{}' does not exist in event-service",
            ticket_cod
        )));
    }

    if !response.status().is_success() {
        return Err(EventServiceError::HttpError(format!(
            "Event-service returned status: {}",
            response.status()
        )));
    }

    Ok(())
}

pub async fn get_ticket_details(
    event_service_url: &str,
    ticket_cod: &str,
) -> Result<TicketDetails, EventServiceError> {
    use tracing::info;

    let ticket_url = format!(
        "{}/api/event-manager/tickets/{}",
        event_service_url, ticket_cod
    );

    info!("Fetching ticket details from: {}", ticket_url);

    let response = reqwest::get(&ticket_url).await.map_err(|e| {
        EventServiceError::HttpError(format!("Failed to connect to event-service: {}", e))
    })?;

    if response.status().as_u16() == 404 {
        return Err(EventServiceError::TicketNotFound(format!(
            "Ticket with code '{}' does not exist in event-service",
            ticket_cod
        )));
    }

    if !response.status().is_success() {
        return Err(EventServiceError::HttpError(format!(
            "Event-service returned status: {}",
            response.status()
        )));
    }

    let ticket_info: TicketInfo = response.json().await.map_err(|e| {
        EventServiceError::DeserializationError(format!("Failed to parse ticket response: {}", e))
    })?;

    info!(
        "Ticket info: event_id={:?}, packet_id={:?}",
        ticket_info.evenimentid, ticket_info.pachetid
    );

    let mut event_info = None;
    let mut packet_info = None;

    if let Some(event_id) = ticket_info.evenimentid {
        let event_url = format!(
            "{}/api/event-manager/events/{}",
            event_service_url, event_id
        );
        info!("Fetching event details from: {}", event_url);

        match reqwest::get(&event_url).await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EventInfo>().await {
                        Ok(event) => {
                            info!("Successfully fetched event: {}", event.nume);
                            event_info = Some(event);
                        }
                        Err(e) => {
                            info!("Failed to parse event response: {}", e);
                        }
                    }
                } else {
                    info!("Event service returned status: {}", response.status());
                }
            }
            Err(e) => {
                info!("Failed to fetch event: {}", e);
            }
        }
    }

    if let Some(packet_id) = ticket_info.pachetid {
        let packet_url = format!(
            "{}/api/event-manager/event-packets/{}",
            event_service_url, packet_id
        );
        info!("Fetching packet details from: {}", packet_url);

        match reqwest::get(&packet_url).await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<PacketInfo>().await {
                        Ok(packet) => {
                            info!("Successfully fetched packet: {}", packet.nume);
                            packet_info = Some(packet);
                        }
                        Err(e) => {
                            info!("Failed to parse packet response: {}", e);
                        }
                    }
                } else {
                    info!("Packet service returned status: {}", response.status());
                }
            }
            Err(e) => {
                info!("Failed to fetch packet: {}", e);
            }
        }
    }

    Ok(TicketDetails {
        ticket: ticket_info,
        event: event_info,
        packet: packet_info,
    })
}

/// Create a new ticket for an event by calling the event-service
/// This will automatically decrement the seat count
pub async fn create_ticket_for_event(
    event_service_url: &str,
    event_id: i32,
) -> Result<TicketDetails, EventServiceError> {
    use tracing::info;

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
            EventServiceError::HttpError(format!("Failed to connect to event-service: {}", e))
        })?;

    let status = response.status();

    if status.as_u16() == 404 {
        return Err(EventServiceError::InvalidReference(format!(
            "Event with ID {} does not exist in event-service",
            event_id
        )));
    }

    if status.as_u16() == 400 {
        return Err(EventServiceError::InvalidReference(format!(
            "Invalid event ID: {}",
            event_id
        )));
    }

    if status.as_u16() == 409 {
        return Err(EventServiceError::NoSeatsAvailable(format!(
            "No seats available for event {}",
            event_id
        )));
    }

    if !status.is_success() {
        return Err(EventServiceError::HttpError(format!(
            "Event-service returned status: {}",
            status
        )));
    }

    // Parse the response to get ticket details
    // The response has ticket fields flattened alongside _links
    #[derive(Deserialize)]
    struct ResponseWrapper {
        #[serde(flatten)]
        ticket: TicketInfo,
    }

    let wrapper: ResponseWrapper = response.json().await.map_err(|e| {
        EventServiceError::DeserializationError(format!("Failed to parse ticket response: {}", e))
    })?;

    let ticket_info: TicketInfo = wrapper.ticket;

    let event_url = format!(
        "{}/api/event-manager/events/{}",
        event_service_url, event_id
    );
    info!("Fetching event details from: {}", event_url);

    let mut event_info = None;
    match reqwest::get(&event_url).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<EventInfo>().await {
                    Ok(event) => {
                        info!("Successfully fetched event: {}", event.nume);
                        event_info = Some(event);
                    }
                    Err(e) => {
                        info!("Failed to parse event response: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            info!("Failed to fetch event: {}", e);
        }
    }

    Ok(TicketDetails {
        ticket: ticket_info,
        event: event_info,
        packet: None,
    })
}

/// Create a new ticket for a packet by calling the event-service
/// This will automatically decrement the seat count
pub async fn create_ticket_for_packet(
    event_service_url: &str,
    packet_id: i32,
) -> Result<TicketDetails, EventServiceError> {
    use tracing::info;

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
            EventServiceError::HttpError(format!("Failed to connect to event-service: {}", e))
        })?;

    let status = response.status();

    if status.as_u16() == 404 {
        return Err(EventServiceError::InvalidReference(format!(
            "Packet with ID {} does not exist in event-service",
            packet_id
        )));
    }

    if status.as_u16() == 400 {
        return Err(EventServiceError::InvalidReference(format!(
            "Invalid packet ID: {}",
            packet_id
        )));
    }

    if status.as_u16() == 409 {
        return Err(EventServiceError::NoSeatsAvailable(format!(
            "No seats available for packet {}",
            packet_id
        )));
    }

    if !status.is_success() {
        return Err(EventServiceError::HttpError(format!(
            "Event-service returned status: {}",
            status
        )));
    }

    // Parse the response to get ticket details
    // The response has ticket fields flattened alongside _links
    #[derive(Deserialize)]
    struct ResponseWrapper {
        #[serde(flatten)]
        ticket: TicketInfo,
    }

    let wrapper: ResponseWrapper = response.json().await.map_err(|e| {
        EventServiceError::DeserializationError(format!("Failed to parse ticket response: {}", e))
    })?;

    let ticket_info: TicketInfo = wrapper.ticket;

    let packet_url = format!(
        "{}/api/event-manager/event-packets/{}",
        event_service_url, packet_id
    );
    info!("Fetching packet details from: {}", packet_url);

    let mut packet_info = None;
    match reqwest::get(&packet_url).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<PacketInfo>().await {
                    Ok(packet) => {
                        info!("Successfully fetched packet: {}", packet.nume);
                        packet_info = Some(packet);
                    }
                    Err(e) => {
                        info!("Failed to parse packet response: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            info!("Failed to fetch packet: {}", e);
        }
    }

    Ok(TicketDetails {
        ticket: ticket_info,
        event: None,
        packet: packet_info,
    })
}

pub async fn delete_ticket(
    event_service_url: &str,
    ticket_cod: &str,
) -> Result<(), EventServiceError> {
    use tracing::info;

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
            EventServiceError::HttpError(format!("Failed to connect to event-service: {}", e))
        })?;

    let status = response.status();

    if status.as_u16() == 404 {
        return Err(EventServiceError::TicketNotFound(format!(
            "Ticket with code '{}' does not exist in event-service",
            ticket_cod
        )));
    }

    if !status.is_success() {
        return Err(EventServiceError::HttpError(format!(
            "Event-service returned status: {}",
            status
        )));
    }

    Ok(())
}
