use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::client::SocialMedia;

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prenume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_info: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_media: Option<SocialMedia>,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    pub success: bool,
    pub token: String,
    pub message: String,
    pub client_id: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub success: bool,
    pub token: String,
    pub message: String,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateRoleRequest {
    pub role: String,
}

#[derive(Serialize, ToSchema)]
pub struct UpdateRoleResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct VerifyEmailRequest {
    pub email: String,
    pub verification_code: String,
}

#[derive(Serialize, ToSchema)]
pub struct VerifyEmailResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ResendVerificationRequest {
    pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct ResendVerificationResponse {
    pub success: bool,
    pub message: String,
}
