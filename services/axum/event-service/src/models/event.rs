use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct Event {
    pub id: i32,
    pub id_owner: i32,
    pub nume: String,
    pub locatie: Option<String>,
    pub descriere: Option<String>,
    #[serde(rename = "numarlocuri")]
    #[sqlx(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct CreateEvent {
    pub id_owner: i32,
    pub nume: String,
    pub locatie: Option<String>,
    pub descriere: Option<String>,
    #[serde(rename = "numarlocuri")]
    #[sqlx(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct UpdateEvent {
    pub id_owner: Option<i32>,
    pub nume: String,
    pub locatie: Option<String>,
    pub descriere: Option<String>,
    #[serde(rename = "numarlocuri")]
    #[sqlx(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}

#[derive(Deserialize)]
pub struct EventQuery {
    #[serde(rename = "location")]
    pub locatie: Option<String>,
    #[serde(rename = "name")]
    pub nume: Option<String>,
}
