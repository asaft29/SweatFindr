use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    pub user_id: i32,
    pub verification_code: String,
}

#[derive(Serialize)]
pub struct VerifyEmailResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct ResendVerificationRequest {
    pub user_id: i32,
    pub email: String,
}

#[derive(Serialize)]
pub struct ResendVerificationResponse {
    pub success: bool,
    pub message: String,
}
