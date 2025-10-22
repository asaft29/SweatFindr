use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct Event {
    pub id: i32,
    pub id_owner: i32,
    pub nume: String,
    pub locatie: String,
    pub descriere: Option<String>,
    #[sqlx(rename = "numarlocuri")]
    #[serde(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEvent {
    pub id_owner: i32,
    pub nume: String,
    pub locatie: String,
    pub descriere: Option<String>,
    #[serde(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateEvent {
    pub nume: String,
    pub locatie: String,
    pub descriere: Option<String>,
    #[serde(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}
