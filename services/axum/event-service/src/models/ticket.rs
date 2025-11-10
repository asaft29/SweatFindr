use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use validator::Validate;
use validator::ValidationError;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Ticket {
    pub cod: String,

    #[sqlx(rename = "pachetid")]
    #[serde(rename = "pachetid")]
    pub id_pachet: Option<i32>,

    #[sqlx(rename = "evenimentid")]
    #[serde(rename = "evenimentid")]
    pub id_event: Option<i32>,
}

#[derive(Debug, Deserialize, FromRow, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_create_ticket"))]
pub struct CreateTicket {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Code must be between 3 and 50 characters"
    ))]
    pub cod: String,

    #[sqlx(rename = "pachetid")]
    #[serde(rename = "pachetid")]
    pub id_pachet: Option<i32>,

    #[sqlx(rename = "evenimentid")]
    #[serde(rename = "evenimentid")]
    pub id_event: Option<i32>,
}

#[derive(Debug, Deserialize, FromRow, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_update_ticket"))]
pub struct UpdateTicket {
    #[sqlx(rename = "pachetid")]
    #[serde(rename = "pachetid")]
    pub id_pachet: Option<i32>,

    #[sqlx(rename = "evenimentid")]
    #[serde(rename = "evenimentid")]
    pub id_event: Option<i32>,
}

fn validate_exclusive_ids(ticket: &impl ExclusiveTicketIds) -> Result<(), ValidationError> {
    match (ticket.get_pachet_id(), ticket.get_event_id()) {
        (Some(_), Some(_)) => {
            let mut err = ValidationError::new("exclusive_ids");
            err.message =
                Some("A ticket can belong to EITHER a packet OR an event, not both.".into());
            Err(err)
        }
        (None, None) => {
            let mut err = ValidationError::new("exclusive_ids");
            err.message = Some("A ticket must belong to a packet OR an event.".into());
            Err(err)
        }
        _ => Ok(()),
    }
}

fn validate_create_ticket(ticket: &CreateTicket) -> Result<(), ValidationError> {
    validate_exclusive_ids(ticket)
}

fn validate_update_ticket(ticket: &UpdateTicket) -> Result<(), ValidationError> {
    validate_exclusive_ids(ticket)
}

trait ExclusiveTicketIds {
    fn get_pachet_id(&self) -> Option<i32>;
    fn get_event_id(&self) -> Option<i32>;
}
impl ExclusiveTicketIds for CreateTicket {
    fn get_pachet_id(&self) -> Option<i32> {
        self.id_pachet
    }
    fn get_event_id(&self) -> Option<i32> {
        self.id_event
    }
}
impl ExclusiveTicketIds for UpdateTicket {
    fn get_pachet_id(&self) -> Option<i32> {
        self.id_pachet
    }
    fn get_event_id(&self) -> Option<i32> {
        self.id_event
    }
}
