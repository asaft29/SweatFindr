use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Bilet {
    #[sqlx(rename = "COD")]
    pub cod: String,

    #[sqlx(rename = "pachetid")]
    pub id_pachet: Option<i32>,

    #[sqlx(rename = "evenimentid")]
    pub id_event: Option<i32>,
}
