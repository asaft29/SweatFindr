use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Ticket {
    pub cod: String,

    #[sqlx(rename = "pachetid")]
    #[serde(rename = "pachetid")]
    pub id_pachet: Option<i32>,

    #[sqlx(rename = "evenimentid")]
    #[serde(rename = "evenimentid")]
    pub id_event: Option<i32>,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct CreateTicket {
    pub cod: String,

    #[sqlx(rename = "pachetid")]
    #[serde(rename = "pachetid")]
    pub id_pachet: Option<i32>,

    #[sqlx(rename = "evenimentid")]
    #[serde(rename = "evenimentid")]
    pub id_event: Option<i32>,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct UpdateTicket {
    #[sqlx(rename = "pachetid")]
    #[serde(rename = "pachetid")]
    pub id_pachet: Option<i32>,

    #[sqlx(rename = "evenimentid")]
    #[serde(rename = "evenimentid")]
    pub id_event: Option<i32>,
}
