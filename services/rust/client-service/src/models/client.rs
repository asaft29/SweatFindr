use bson::oid::ObjectId;
use common::authorization::{ResourceOwnership, UserClaims};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Client {
    #[serde(rename = "_id")]
    #[schema(value_type = String)]
    pub id: ObjectId,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prenume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_info: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_media: Option<SocialMedia>,
    #[serde(default)]
    pub lista_bilete: Vec<TicketRef>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct SocialMedia {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facebook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instagram: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linkedin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct TicketRef {
    pub cod: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nume_eveniment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locatie: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriere: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateClient {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(
        min = 2,
        max = 50,
        message = "Prenume must be between 2 and 50 characters"
    ))]
    pub prenume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(
        min = 2,
        max = 50,
        message = "Nume must be between 2 and 50 characters"
    ))]
    pub nume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_info: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_media: Option<SocialMedia>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateClient {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(
        min = 2,
        max = 50,
        message = "Prenume must be between 2 and 50 characters"
    ))]
    pub prenume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(
        min = 2,
        max = 50,
        message = "Nume must be between 2 and 50 characters"
    ))]
    pub nume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_info: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_media: Option<SocialMedia>,
}

#[derive(Debug, Deserialize, ToSchema, Clone)]
pub struct ClientQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prenume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nume: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_add_ticket"))]
pub struct AddTicket {
    #[serde(rename = "evenimentid")]
    pub id_event: Option<i32>,
    #[serde(rename = "pachetid")]
    pub id_pachet: Option<i32>,
}

fn validate_add_ticket(ticket: &AddTicket) -> Result<(), validator::ValidationError> {
    match (ticket.id_event, ticket.id_pachet) {
        (Some(_), Some(_)) => {
            let mut err = validator::ValidationError::new("exclusive_ids");
            err.message = Some("A ticket can be for EITHER an event OR a packet, not both.".into());
            Err(err)
        }
        (None, None) => {
            let mut err = validator::ValidationError::new("exclusive_ids");
            err.message = Some("Must specify either evenimentid or pachetid.".into());
            Err(err)
        }
        (Some(event_id), None) => {
            if event_id < 0 {
                let mut err = validator::ValidationError::new("negative_id");
                err.message = Some("Event ID cannot be negative".into());
                return Err(err);
            }
            Ok(())
        }
        (None, Some(packet_id)) => {
            if packet_id < 0 {
                let mut err = validator::ValidationError::new("negative_id");
                err.message = Some("Packet ID cannot be negative".into());
                return Err(err);
            }
            Ok(())
        }
    }
}

impl ResourceOwnership for Client {
    fn is_owned_by(&self, claims: &UserClaims, user_email: Option<&str>) -> bool {
        if let Some(email) = user_email {
            claims.is_client() && self.email == email
        } else {
            false
        }
    }

    fn resource_description(&self) -> String {
        "client profile".to_string()
    }
}
