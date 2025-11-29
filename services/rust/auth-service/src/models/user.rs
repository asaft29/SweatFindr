use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub parola: String,
    pub rol: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum UserRole {
    Admin,
    OwnerEvent,
    Client,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User : {}", self.email)
    }
}

impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let role_str = match self {
            UserRole::Admin => "admin",
            UserRole::OwnerEvent => "owner-event",
            UserRole::Client => "client",
        };
        write!(f, "{}", role_str)
    }
}

impl FromStr for UserRole {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(UserRole::Admin),
            "owner-event" => Ok(UserRole::OwnerEvent),
            "client" => Ok(UserRole::Client),
            other => Err(anyhow!("Invalid role : {}", other)),
        }
    }
}
