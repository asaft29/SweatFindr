use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EventPackets {
    pub id: i32,
    pub id_owner: i32,
    pub nume: String,
    pub locatie: Option<String>,
    pub descriere: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventPacket {
    pub id_owner: i32,
    pub nume: String,
    pub locatie: Option<String>,
    pub descriere: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEventPacket {
    pub id_owner: Option<i32>,
    pub nume: String,
    pub locatie: Option<String>,
    pub descriere: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EventPacketQuery {
    #[serde(rename = "type")]
    pub descriere: Option<String>,
    #[serde(rename = "available_tickets")]
    pub bilete: Option<i32>,
    #[serde(flatten)]
    pub paginare: PaginationParams,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub page: Option<i64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub items_per_page: Option<i64>,
}
