use reqwest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug)]
pub enum EventServiceError {
    TicketNotFound(String),
    HttpError(String),
    DeserializationError(String),
}

impl std::fmt::Display for EventServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TicketNotFound(msg) => write!(f, "{}", msg),
            Self::HttpError(msg) => write!(f, "{}", msg),
            Self::DeserializationError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for EventServiceError {}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TicketInfoRaw {
    cod: String,
    pachetid: Option<i32>,
    evenimentid: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TicketInfo {
    pub cod: String,
    pub pachetid: Option<i32>,
    pub evenimentid: Option<i32>,
}

impl From<TicketInfoRaw> for TicketInfo {
    fn from(raw: TicketInfoRaw) -> Self {
        Self {
            cod: raw.cod,
            pachetid: raw.pachetid,
            evenimentid: raw.evenimentid,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EventInfoRaw {
    id: i32,
    id_owner: i32,
    nume: String,
    locatie: String,
    descriere: String,
    numarlocuri: i32,
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

impl From<EventInfoRaw> for EventInfo {
    fn from(raw: EventInfoRaw) -> Self {
        Self {
            id: raw.id,
            id_owner: raw.id_owner,
            nume: raw.nume,
            locatie: raw.locatie,
            descriere: raw.descriere,
            numarlocuri: raw.numarlocuri,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PacketInfoRaw {
    id: i32,
    id_owner: i32,
    nume: String,
    locatie: String,
    descriere: String,
    numarlocuri: i32,
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

impl From<PacketInfoRaw> for PacketInfo {
    fn from(raw: PacketInfoRaw) -> Self {
        Self {
            id: raw.id,
            id_owner: raw.id_owner,
            nume: raw.nume,
            locatie: raw.locatie,
            descriere: raw.descriere,
            numarlocuri: raw.numarlocuri,
        }
    }
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

    let ticket_raw: TicketInfoRaw = response.json().await.map_err(|e| {
        EventServiceError::DeserializationError(format!("Failed to parse ticket response: {}", e))
    })?;
    let ticket_info: TicketInfo = ticket_raw.into();

    info!("Ticket info: event_id={:?}, packet_id={:?}", ticket_info.evenimentid, ticket_info.pachetid);

    let mut event_info = None;
    let mut packet_info = None;

    if let Some(event_id) = ticket_info.evenimentid {
        let event_url = format!("{}/api/event-manager/events/{}", event_service_url, event_id);
        info!("Fetching event details from: {}", event_url);

        match reqwest::get(&event_url).await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EventInfoRaw>().await {
                        Ok(event_raw) => {
                            info!("Successfully fetched event: {}", event_raw.nume);
                            event_info = Some(event_raw.into());
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
                    match response.json::<PacketInfoRaw>().await {
                        Ok(packet_raw) => {
                            info!("Successfully fetched packet: {}", packet_raw.nume);
                            packet_info = Some(packet_raw.into());
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
