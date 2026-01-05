use crate::repository::verification_repository::VerificationRepository;
use crate::services::email_service::EmailService;
use std::sync::Arc;
use tonic::{Request, Response, Status};
pub mod email {
    tonic::include_proto!("email");
}

pub mod auth {
    tonic::include_proto!("auth");
}

use auth::auth_service_client::AuthServiceClient;
use auth::{GetUserIdByEmailRequest, MarkEmailVerifiedRequest};
use email::email_service_server::EmailService as EmailServiceTrait;
use email::{
    ResendVerificationRequest, ResendVerificationResponse, SendPasswordResetRequest,
    SendPasswordResetResponse, SendVerificationRequest, SendVerificationResponse,
    ValidateResetTokenRequest, ValidateResetTokenResponse, VerifyCodeRequest, VerifyCodeResponse,
    VerifyPasswordResetCodeRequest, VerifyPasswordResetCodeResponse,
};

pub struct EmailServiceImpl {
    email_service: Arc<EmailService>,
    verification_repo: Arc<VerificationRepository>,
    auth_service_url: String,
}

impl EmailServiceImpl {
    pub fn new(
        email_service: Arc<EmailService>,
        verification_repo: Arc<VerificationRepository>,
        auth_service_url: String,
    ) -> Self {
        Self {
            email_service,
            verification_repo,
            auth_service_url,
        }
    }
}

#[tonic::async_trait]
impl EmailServiceTrait for EmailServiceImpl {
    async fn send_verification_email(
        &self,
        request: Request<SendVerificationRequest>,
    ) -> Result<Response<SendVerificationResponse>, Status> {
        let req = request.into_inner();

        let verification_code = EmailService::generate_verification_code();

        self.verification_repo
            .create_verification(req.user_id, &verification_code)
            .await
            .map_err(|e| Status::internal(format!("Failed to store verification code: {}", e)))?;

        self.email_service
            .send_verification_email(&req.email, &verification_code)
            .await
            .map_err(|e| Status::internal(format!("Failed to send verification email: {}", e)))?;

        Ok(Response::new(SendVerificationResponse {
            success: true,
            message: "Verification email sent successfully".to_string(),
        }))
    }

    async fn verify_code(
        &self,
        request: Request<VerifyCodeRequest>,
    ) -> Result<Response<VerifyCodeResponse>, Status> {
        let req = request.into_inner();

        let verified = self
            .verification_repo
            .verify_code(req.user_id, &req.verification_code)
            .await
            .map_err(|e| Status::internal(format!("Failed to verify code: {}", e)))?;

        if !verified {
            return Ok(Response::new(VerifyCodeResponse {
                success: false,
                message: "Invalid or expired verification code".to_string(),
            }));
        }

        let mut auth_client = AuthServiceClient::connect(self.auth_service_url.clone())
            .await
            .map_err(|e| Status::internal(format!("Failed to connect to auth service: {}", e)))?;

        match auth_client
            .mark_email_verified(Request::new(MarkEmailVerifiedRequest {
                user_id: req.user_id,
            }))
            .await
        {
            Ok(_) => Ok(Response::new(VerifyCodeResponse {
                success: true,
                message: "Email verified successfully".to_string(),
            })),
            Err(e) => Err(Status::internal(format!(
                "Failed to mark email as verified: {}",
                e
            ))),
        }
    }

    async fn resend_verification_code(
        &self,
        request: Request<ResendVerificationRequest>,
    ) -> Result<Response<ResendVerificationResponse>, Status> {
        let req = request.into_inner();

        let verification_code = EmailService::generate_verification_code();

        self.verification_repo
            .create_verification(req.user_id, &verification_code)
            .await
            .map_err(|e| Status::internal(format!("Failed to store verification code: {}", e)))?;

        self.email_service
            .send_verification_email(&req.email, &verification_code)
            .await
            .map_err(|e| Status::internal(format!("Failed to send verification email: {}", e)))?;

        Ok(Response::new(ResendVerificationResponse {
            success: true,
            message: "Verification code resent successfully".to_string(),
        }))
    }

    async fn send_password_reset_email(
        &self,
        request: Request<SendPasswordResetRequest>,
    ) -> Result<Response<SendPasswordResetResponse>, Status> {
        let req = request.into_inner();

        let user_exists = match AuthServiceClient::connect(self.auth_service_url.clone()).await {
            Ok(mut auth_client) => {
                match auth_client
                    .get_user_id_by_email(Request::new(GetUserIdByEmailRequest {
                        email: req.email.clone(),
                    }))
                    .await
                {
                    Ok(response) => response.into_inner().success,
                    Err(e) => {
                        tracing::error!("Failed to check if user exists: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to connect to auth service: {}", e);
                false
            }
        };

        if !user_exists {
            return Ok(Response::new(SendPasswordResetResponse { success: true }));
        }

        let reset_code = EmailService::generate_verification_code();

        if let Err(e) = self
            .verification_repo
            .create_password_reset(&req.email, &reset_code)
            .await
        {
            tracing::error!("Failed to store reset code: {}", e);
        }

        let email_service = Arc::clone(&self.email_service);
        let email_for_spawn = req.email.clone(); 
        tokio::spawn(async move {
            if let Err(e) = email_service
                .send_password_reset_email(&email_for_spawn, &reset_code)
                .await
            {
                tracing::error!("Failed to send password reset email: {}", e);
            }
        });

        Ok(Response::new(SendPasswordResetResponse { success: true }))
    }

    async fn verify_password_reset_code(
        &self,
        request: Request<VerifyPasswordResetCodeRequest>,
    ) -> Result<Response<VerifyPasswordResetCodeResponse>, Status> {
        let req = request.into_inner();

        let reset_token = self
            .verification_repo
            .verify_password_reset_code(&req.email, &req.reset_code)
            .await
            .map_err(|e| Status::internal(format!("Failed to verify reset code: {}", e)))?;

        match reset_token {
            Some(token) => Ok(Response::new(VerifyPasswordResetCodeResponse {
                success: true,
                message: "Reset code verified successfully".to_string(),
                reset_token: token,
            })),
            None => Ok(Response::new(VerifyPasswordResetCodeResponse {
                success: false,
                message: "Invalid or expired reset code".to_string(),
                reset_token: String::new(),
            })),
        }
    }

    async fn validate_reset_token(
        &self,
        request: Request<ValidateResetTokenRequest>,
    ) -> Result<Response<ValidateResetTokenResponse>, Status> {
        let req = request.into_inner();

        let valid = self
            .verification_repo
            .validate_reset_token(&req.email, &req.reset_token)
            .await
            .map_err(|e| Status::internal(format!("Failed to validate reset token: {}", e)))?;

        Ok(Response::new(ValidateResetTokenResponse { valid }))
    }
}
