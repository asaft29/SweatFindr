use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Event {
    #[sqlx(rename = "ID")]
    pub id: i32,

    #[sqlx(rename = "ID_OWNER")]
    pub owner_id: i32,

    #[sqlx(rename = "nume")]
    pub name: String,

    #[sqlx(rename = "locatie")]
    pub location: String,

    #[sqlx(rename = "descriere")]
    pub description: Option<String>,

    #[sqlx(rename = "numarlocuri")]
    pub sit_count: Option<i32>,
}
