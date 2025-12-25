use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub message: String,
    pub user_id: Option<i32>,
    pub token_value: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub token_value: Option<String>,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub token_value: String,
}

#[derive(Serialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}
