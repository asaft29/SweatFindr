use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct JoinPe {
    #[sqlx(rename = "pachetid")]
    pub id_pachet: i32,

    #[sqlx(rename = "evenimentid")]
    pub id_eveniment: i32,

    #[sqlx(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}
