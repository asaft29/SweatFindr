use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use common::links::HateoasRequest;

#[derive(Debug)]
pub enum ExternalServiceError {
    NotFound(String),
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
