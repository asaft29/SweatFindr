use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct EventPacketRelation {
    #[sqlx(rename = "pachetid")]
    #[serde(rename = "pachetid")]
    pub id_pachet: i32,
    #[sqlx(rename = "evenimentid")]
    #[serde(rename = "evenimentid")]
    pub id_event: i32,
    #[sqlx(rename = "numarlocuri")]
    #[serde(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct AddEventToPacket {
    #[sqlx(rename = "pachetid")]
    #[serde(rename = "pachetid")]
    pub id_pachet: i32,
    #[sqlx(rename = "numarlocuri")]
    #[serde(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct AddPacketToEvent {
    #[sqlx(rename = "evenimentid")]
    #[serde(rename = "evenimentid")]
    pub id_event: i32,
    #[sqlx(rename = "numarlocuri")]
    #[serde(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}
