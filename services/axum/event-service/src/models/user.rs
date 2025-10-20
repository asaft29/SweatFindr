use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    #[sqlx(rename = "ID")]
    pub id: i32,

    pub email: String,
    #[sqlx(rename = "parola")]
    pub password: String,

    #[sqlx(rename = "rol")]
    pub role: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Role {
    Admin,
    OwnerEvent,
    Client,
}
