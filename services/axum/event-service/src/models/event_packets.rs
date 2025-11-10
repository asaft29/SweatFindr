use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct EventPackets {
    pub id: i32,
    pub id_owner: i32,
    pub nume: String,
    pub locatie: Option<String>,
    pub descriere: Option<String>,
    pub numarlocuri: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateEventPacket {
    pub id_owner: i32,
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
    pub numarlocuri: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateEventPacket {
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
    pub numarlocuri: Option<i32>,
}

#[derive(Debug, Deserialize, Clone, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct EventPacketQuery {
    #[serde(rename = "type")]
    #[validate(length(
        max = 50,
        message = "Description filter must be less than 50 characters"
    ))]
    pub descriere: Option<String>,
    #[validate(range(min = 1, message = "Available tickets must be at least 1"))]
    #[serde(rename = "available_tickets")]
    pub bilete: Option<i32>,
    #[serde(flatten)]
    pub paginare: PaginationParams,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone, ToSchema, Validate)]
pub struct PaginationParams {
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub page: Option<i64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[validate(range(
        min = 1,
        max = 100,
        message = "items_per_page must be between 1 and 100"
    ))]
    pub items_per_page: Option<i64>,
}
