use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Package {
    pub id: i32,

    pub id_owner: i32,

    pub nume: String,

    pub locatie: String,

    pub descriere: Option<String>,
}
