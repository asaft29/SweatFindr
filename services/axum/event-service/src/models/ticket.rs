use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Bilet {
    #[sqlx(rename = "COD")]
    pub code: String,

    #[sqlx(rename = "PachetID")]
    pub pachet_id: Option<i32>,

    #[sqlx(rename = "EvenimentID")]
    pub event_id: Option<i32>,
}
