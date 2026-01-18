use crate::models::UserRole;
use crate::repository::UserRepository;
use crate::services::{JwtService, TokenBlacklist};
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub mod auth {
    tonic::include_proto!("auth");
}

pub mod email {
    tonic::include_proto!("email");
}

use email::email_service_client::EmailServiceClient as EmailClient;

use auth::auth_service_server::AuthService;
use auth::*;

pub struct AuthServiceImpl {
    pub user_repo: Arc<UserRepository>,
    pub jwt_service: Arc<JwtService>,
    pub blacklist: TokenBlacklist,
    pub email_service_url: String,
    pub client_service_url: String,
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn authenticate(
        &self,
        request: Request<AuthRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        let user = match self.user_repo.find_by_email(&req.username).await {
            Ok(Some(user)) => user,
            Ok(_) => {
                return Err(Status::unauthenticated("Invalid credentials"));
            }
            Err(e) => {
                return Err(Status::internal(format!("Database error: {}", e)));
            }
        };

        match bcrypt::verify(&req.password, &user.parola) {
            Ok(true) => {}
            Ok(false) => {
                return Err(Status::unauthenticated("Invalid credentials"));
            }
            Err(e) => {
                return Err(Status::internal(format!(
                    "Password verification error: {}",
                    e
                )));
            }
        }

        if !user.email_verified {
            return Ok(Response::new(AuthResponse {
                success: false,
                token_value: String::new(),
                message: "Email not verified. Please verify your email before logging in."
                    .to_string(),
            }));
        }

        self.blacklist.clear_user_invalidation(user.id).await;

        let token = match self
            .jwt_service
            .generate_token(user.id, &user.rol.to_string())
        {
            Ok(token) => token,
            Err(e) => {
                return Err(Status::internal(format!("Token generation error: {}", e)));
            }
        };

        Ok(Response::new(AuthResponse {
            success: true,
            token_value: token,
            message: "Authentication successful".to_string(),
        }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateRequest>,
    ) -> Result<Response<ValidateResponse>, Status> {
        let req = request.into_inner();

        if self.blacklist.contains(&req.token_value).await {
            return Ok(Response::new(ValidateResponse {
                success: true,
                valid: false,
                user_id: 0,
                role: String::new(),
                message: "Token has been invalidated or expired".to_string(),
            }));
        }

        match self.jwt_service.validate_token(&req.token_value) {
            Ok(claims) => {
                let now = chrono::Utc::now().timestamp();
                if claims.exp < now {
                    self.blacklist.add(req.token_value).await;

                    return Ok(Response::new(ValidateResponse {
                        success: true,
                        valid: false,
                        user_id: 0,
                        role: String::new(),
                        message: "Token has expired".to_string(),
                    }));
                }

                if self.blacklist.is_user_invalidated(claims.sub).await {
                    return Ok(Response::new(ValidateResponse {
                        success: true,
                        valid: false,
                        user_id: 0,
                        role: String::new(),
                        message: "User session invalidated. Please login again.".to_string(),
                    }));
                }

                Ok(Response::new(ValidateResponse {
                    success: true,
                    valid: true,
                    user_id: claims.sub,
                    role: claims.role,
                    message: "Token is valid".to_string(),
                }))
            }
            Err(e) => {
                self.blacklist.add(req.token_value).await;

                Ok(Response::new(ValidateResponse {
                    success: true,
                    valid: false,
                    user_id: 0,
                    role: String::new(),
                    message: format!("Token validation failed: {}", e),
                }))
            }
        }
    }

    async fn destroy_token(
        &self,
        request: Request<DestroyRequest>,
    ) -> Result<Response<DestroyResponse>, Status> {
        let req = request.into_inner();

        match self.jwt_service.validate_token(&req.token_value) {
            Ok(_) => {
                self.blacklist.add(req.token_value).await;

                Ok(Response::new(DestroyResponse {
                    success: true,
                    message: "Token successfully destroyed".to_string(),
                }))
            }
            Err(_) => {
                self.blacklist.add(req.token_value).await;

                Ok(Response::new(DestroyResponse {
                    success: true,
                    message: "Token destroyed (was already invalid)".to_string(),
                }))
            }
        }
    }

    async fn register_user(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();

        if email_address::EmailAddress::from_str(&req.email).is_err() {
            return Err(Status::invalid_argument("Invalid email format"));
        }

        if let Some(domain) = req.email.split('@').nth(1) {
            if !domain.contains('.') {
                return Err(Status::invalid_argument(
                    "Invalid email domain - must include domain name and TLD (e.g., example.com)",
                ));
            }

            let tld_part = domain.rsplit('.').next().unwrap_or("");
            if !tld::exist(tld_part) {
                return Err(Status::invalid_argument(
                    "Invalid email domain - TLD does not exist",
                ));
            }
        }

        match self.user_repo.find_by_email(&req.email).await {
            Ok(Some(_)) => {
                return Err(Status::already_exists("Email already registered"));
            }
            Ok(_) => {}
            Err(e) => {
                return Err(Status::internal(format!("Database error: {}", e)));
            }
        }

        let role = match UserRole::from_str(&req.role) {
            Ok(role) => role,
            Err(_) => {
                return Err(Status::invalid_argument(
                    "Invalid role. Must be 'admin', 'owner-event', or 'client'",
                ));
            }
        };

        let hashed_password = match bcrypt::hash(&req.password, bcrypt::DEFAULT_COST) {
            Ok(hash) => hash,
            Err(e) => {
                return Err(Status::internal(format!("Password hashing error: {}", e)));
            }
        };

        let user_id = match self
            .user_repo
            .create_user(&req.email, &hashed_password, &role)
            .await
        {
            Ok(id) => id,
            Err(e) => {
                return Err(Status::internal(format!("Failed to create user: {}", e)));
            }
        };

        let token = match self.jwt_service.generate_token(user_id, &role.to_string()) {
            Ok(token) => token,
            Err(e) => {
                return Err(Status::internal(format!("Token generation error: {}", e)));
            }
        };

        let client_service_url = self.client_service_url.clone();
        let client_email = req.email.clone();
        let service_token = token.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let create_client_payload = serde_json::json!({
                "email": client_email
            });

            match client
                .post(format!("{}/api/client-manager/clients", client_service_url))
                .header("Authorization", format!("Bearer {}", service_token))
                .header("Content-Type", "application/json")
                .json(&create_client_payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        tracing::info!("Client profile created for user {}", client_email);
                    } else {
                        tracing::error!(
                            "Failed to create client profile: HTTP {}",
                            response.status()
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to call client-service: {}", e);
                }
            }
        });

        let email_service_url = self.email_service_url.clone();
        let user_email = req.email.clone();
        tokio::spawn(async move {
            match EmailClient::connect(email_service_url).await {
                Ok(mut client) => {
                    let email_request = email::SendVerificationRequest {
                        user_id,
                        email: user_email,
                    };

                    match client.send_verification_email(email_request).await {
                        Ok(_) => {
                            tracing::info!("Verification email sent to user {}", user_id);
                        }
                        Err(e) => {
                            tracing::error!("Failed to send verification email: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to connect to email service: {}", e);
                }
            }
        });

        Ok(Response::new(RegisterResponse {
            success: true,
            user_id,
            token_value: token,
            message: "User registered successfully. Check your email for verification code."
                .to_string(),
        }))
    }

    async fn get_user_email(
        &self,
        request: Request<GetUserEmailRequest>,
    ) -> Result<Response<GetUserEmailResponse>, Status> {
        let user_id = request.into_inner().user_id;
        match self.user_repo.find_by_id(user_id).await {
            Ok(Some(user)) => Ok(Response::new(GetUserEmailResponse {
                success: true,
                email: user.email,
                message: "Email found".to_string(),
            })),
            Ok(_) => Err(Status::not_found(format!(
                "User with id {} not found",
                user_id
            ))),
            Err(e) => Err(Status::internal(format!("Database error: {}", e))),
        }
    }

    async fn update_role(
        &self,
        request: Request<UpdateRoleRequest>,
    ) -> Result<Response<UpdateRoleResponse>, Status> {
        let req = request.into_inner();
        let user_id = req.user_id;
        let role_str = req.role.to_lowercase();
        use crate::models::UserRole;
        use std::str::FromStr;

        let role = match UserRole::from_str(&role_str) {
            Ok(role) => role,
            Err(_) => {
                return Err(Status::invalid_argument(format!(
                    "Invalid role '{}'. Must be one of: admin, client, owner-event",
                    req.role
                )));
            }
        };

        match self.user_repo.find_by_id(user_id).await {
            Ok(Some(_)) => match self.user_repo.update_role(user_id, &role).await {
                Ok(true) => {
                    self.blacklist.invalidate_user(user_id).await;

                    Ok(Response::new(UpdateRoleResponse {
                            success: true,
                            message: format!(
                                "User {} role updated to {} successfully. All existing sessions invalidated.",
                                user_id, role_str
                            ),
                        }))
                }
                Ok(false) => Err(Status::not_found(format!(
                    "User with id {} not found",
                    user_id
                ))),
                Err(e) => Err(Status::internal(format!("Database error: {}", e))),
            },
            Ok(_) => Err(Status::not_found(format!(
                "User with id {} not found",
                user_id
            ))),
            Err(e) => Err(Status::internal(format!("Database error: {}", e))),
        }
    }

    async fn get_user_id_by_email(
        &self,
        request: Request<GetUserIdByEmailRequest>,
    ) -> Result<Response<GetUserIdByEmailResponse>, Status> {
        let req = request.into_inner();

        match self.user_repo.find_by_email(&req.email).await {
            Ok(Some(user)) => Ok(Response::new(GetUserIdByEmailResponse {
                success: true,
                user_id: user.id,
                message: "User found".to_string(),
                email_verified: user.email_verified,
            })),
            Ok(_) => Err(Status::not_found("User not found")),
            Err(e) => Err(Status::internal(format!("Database error: {}", e))),
        }
    }

    async fn mark_email_verified(
        &self,
        request: Request<MarkEmailVerifiedRequest>,
    ) -> Result<Response<MarkEmailVerifiedResponse>, Status> {
        let req = request.into_inner();

        match self.user_repo.mark_email_verified(req.user_id).await {
            Ok(true) => Ok(Response::new(MarkEmailVerifiedResponse {
                success: true,
                message: "Email marked as verified".to_string(),
            })),
            Ok(false) => Err(Status::not_found("User not found")),
            Err(e) => Err(Status::internal(format!("Database error: {}", e))),
        }
    }

    async fn delete_unverified_user(
        &self,
        request: Request<DeleteUnverifiedUserRequest>,
    ) -> Result<Response<DeleteUnverifiedUserResponse>, Status> {
        let req = request.into_inner();

        match self.user_repo.delete_unverified_user(req.user_id).await {
            Ok(true) => Ok(Response::new(DeleteUnverifiedUserResponse {
                success: true,
                message: format!("Unverified user {} deleted successfully", req.user_id),
            })),
            Ok(false) => Err(Status::not_found(
                "User not found or already verified (verified users cannot be deleted via this method)",
            )),
            Err(e) => Err(Status::internal(format!("Database error: {}", e))),
        }
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();

        match self.user_repo.delete_user(req.user_id).await {
            Ok(true) => {
                self.blacklist.invalidate_user(req.user_id).await;
                Ok(Response::new(DeleteUserResponse {
                    success: true,
                    message: format!("User {} deleted successfully", req.user_id),
                }))
            }
            Ok(false) => Err(Status::not_found("User not found")),
            Err(e) => Err(Status::internal(format!("Database error: {}", e))),
        }
    }

    async fn reset_password(
        &self,
        request: Request<ResetPasswordRequest>,
    ) -> Result<Response<ResetPasswordResponse>, Status> {
        let req = request.into_inner();

        let mut email_client = EmailClient::connect(self.email_service_url.clone())
            .await
            .map_err(|e| Status::internal(format!("Failed to connect to email service: {}", e)))?;

        let token_response = email_client
            .validate_reset_token(email::ValidateResetTokenRequest {
                email: req.email.clone(),
                reset_token: req.reset_token.clone(),
            })
            .await
            .map_err(|e| Status::internal(format!("Failed to validate reset token: {}", e)))?;

        if !token_response.into_inner().valid {
            return Err(Status::permission_denied(
                "Invalid or expired reset token. Please verify your code again.",
            ));
        }

        let user = match self.user_repo.find_by_email(&req.email).await {
            Ok(Some(user)) => user,
            Ok(_) => {
                return Err(Status::not_found("User not found"));
            }
            Err(e) => {
                return Err(Status::internal(format!("Database error: {}", e)));
            }
        };

        let hashed_password = match bcrypt::hash(&req.new_password, bcrypt::DEFAULT_COST) {
            Ok(hash) => hash,
            Err(e) => {
                return Err(Status::internal(format!("Password hashing error: {}", e)));
            }
        };

        match self
            .user_repo
            .update_password(user.id, &hashed_password)
            .await
        {
            Ok(true) => {
                self.blacklist.invalidate_user(user.id).await;

                Ok(Response::new(ResetPasswordResponse {
                    success: true,
                    message: "Password reset successfully. Please login with your new password."
                        .to_string(),
                }))
            }
            Ok(false) => Err(Status::not_found("User not found")),
            Err(e) => Err(Status::internal(format!("Database error: {}", e))),
        }
    }
}
