use common::authorization::{ResourceOwnership, UserClaims};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct Event {
    pub id: i32,
    pub id_owner: i32,
    pub nume: String,
    pub locatie: Option<String>,
    pub descriere: Option<String>,
    #[serde(rename = "numarlocuri")]
    #[sqlx(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}

impl ResourceOwnership for Event {
    fn is_owned_by(&self, claims: &UserClaims, _user_email: Option<&str>) -> bool {
        claims.is_owner_event() && self.id_owner == claims.user_id
    }

    fn resource_description(&self) -> String {
        format!("event '{}'", self.nume)
    }
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateEvent {
    #[validate(length(
        min = 3,
        max = 100,
        message = "Name must be between 3 and 100 characters"
    ))]
    pub nume: String,
    #[validate(length(max = 255, message = "Location must be less than 255 characters"))]
    pub locatie: Option<String>,
    #[validate(length(
        min = 10,
        max = 500,
        message = "Description must be between 10 and 500 characters"
    ))]
    pub descriere: Option<String>,
    #[validate(range(min = 1, max = 50000, message = "Seats must be between 1 and 50,000"))]
    #[serde(rename = "numarlocuri")]
    #[sqlx(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateEvent {
    pub id_owner: Option<i32>,
    #[validate(length(
        min = 3,
        max = 100,
        message = "Name must be between 3 and 100 characters"
    ))]
    pub nume: String,
    #[validate(length(max = 255, message = "Location must be less than 255 characters"))]
    pub locatie: Option<String>,
    #[validate(length(
        min = 10,
        max = 500,
        message = "Description must be between 10 and 500 characters"
    ))]
    pub descriere: Option<String>,
    #[validate(range(min = 1, max = 50000, message = "Seats must be between 1 and 50,000"))]
    #[serde(rename = "numarlocuri")]
    #[sqlx(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct PatchEvent {
    pub id_owner: Option<i32>,
    #[validate(length(
        min = 3,
        max = 100,
        message = "Name must be between 3 and 100 characters"
    ))]
    pub nume: Option<String>,
    #[validate(length(max = 255, message = "Location must be less than 255 characters"))]
    pub locatie: Option<String>,
    #[validate(length(
        min = 10,
        max = 500,
        message = "Description must be between 10 and 500 characters"
    ))]
    pub descriere: Option<String>,
    #[validate(range(min = 1, max = 50000, message = "Seats must be between 1 and 50,000"))]
    #[serde(rename = "numarlocuri")]
    pub locuri: Option<i32>,
}

#[derive(Deserialize, Clone, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct EventQuery {
    #[validate(length(max = 50, message = "Location filter must be less than 50 characters"))]
    #[serde(rename = "location")]
    pub locatie: Option<String>,
    #[validate(length(max = 50, message = "Name filter must be less than 50 characters"))]
    #[serde(rename = "name")]
    pub nume: Option<String>,
}
