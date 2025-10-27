use serde::{Deserialize, Serialize};
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
