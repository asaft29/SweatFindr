use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub parola: String,
    pub rol: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Role {
    Admin,
    OwnerEvent,
    Client,
}
