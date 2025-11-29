use tonic::{Request, Response, Status};

use crate::repository::UserRepository;
use crate::services::{JwtService, TokenBlacklist};

pub mod auth {
    tonic::include_proto!("auth");
}

use auth::auth_service_server::AuthService;
use auth::{
    AuthRequest, AuthResponse, DestroyRequest, DestroyResponse, RegisterRequest, RegisterResponse,
    ValidateRequest, ValidateResponse,
};

pub struct AuthServiceImpl {
    pub user_repo: UserRepository,
    pub jwt_service: JwtService,
    pub blacklist: TokenBlacklist,
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
                return Ok(Response::new(AuthResponse {
                    success: false,
                    token_value: String::new(),
                    message: "Invalid credentials".to_string(),
                }));
            }
            Err(e) => {
                return Err(Status::internal(format!("Database error: {}", e)));
            }
        };

        match bcrypt::verify(&req.password, &user.parola) {
            Ok(true) => {}
            Ok(false) => {
                return Ok(Response::new(AuthResponse {
                    success: false,
                    token_value: String::new(),
                    message: "Invalid credentials".to_string(),
                }));
            }
            Err(e) => {
                return Err(Status::internal(format!(
                    "Password verification error: {}",
                    e
                )));
            }
        }

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

        if !req.email.contains('@') {
            return Err(Status::invalid_argument("Invalid email format"));
        }

        match self.user_repo.find_by_email(&req.email).await {
            Ok(Some(_)) => {
                return Err(Status::already_exists("Email already registered"));
            }
            Ok(None) => {}
            Err(e) => {
                return Err(Status::internal(format!("Database error: {}", e)));
            }
        }

        use crate::models::UserRole;
        use std::str::FromStr;

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

        Ok(Response::new(RegisterResponse {
            success: true,
            user_id,
            token_value: token,
            message: "User registered successfully".to_string(),
        }))
    }
}
