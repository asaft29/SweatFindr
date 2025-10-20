use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct JoinPe {
    #[sqlx(rename = "PachetID")]
    pub pachet_id: i32,

    #[sqlx(rename = "EvenimentID")]
    pub eveniment_id: i32,

    #[sqlx(rename = "numarLocuri")]
    pub numar_locuri: Option<i32>,
}
