use bson::oid::ObjectId;
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
    pub public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct TicketRef {
    pub cod: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nume_eveniment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locatie: Option<String>,
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

#[derive(Debug, Deserialize, ToSchema)]
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
pub struct AddTicket {
    #[validate(length(min = 1, message = "Ticket code cannot be empty"))]
    pub cod: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nume_eveniment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locatie: Option<String>,
}
