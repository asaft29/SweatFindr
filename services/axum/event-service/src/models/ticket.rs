use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Ticket {
    pub cod: String,

    #[sqlx(rename = "pachetid")]
    #[serde(rename = "pachetid")]
    pub id_pachet: Option<i32>,

    #[sqlx(rename = "evenimentid")]
    #[serde(rename = "evenimentid")]
    pub id_event: Option<i32>,
}

#[derive(Debug, Deserialize, FromRow, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateTicket {
    pub cod: String,

    #[sqlx(rename = "pachetid")]
    #[serde(rename = "pachetid")]
    pub id_pachet: Option<i32>,

    #[sqlx(rename = "evenimentid")]
    #[serde(rename = "evenimentid")]
    pub id_event: Option<i32>,
}

#[derive(Debug, Deserialize, FromRow, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateTicket {
    #[sqlx(rename = "pachetid")]
    #[serde(rename = "pachetid")]
    pub id_pachet: Option<i32>,

    #[sqlx(rename = "evenimentid")]
    #[serde(rename = "evenimentid")]
    pub id_event: Option<i32>,
}
