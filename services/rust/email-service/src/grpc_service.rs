use crate::repository::verification_repository::VerificationRepository;
use crate::services::email_service::EmailService;
use tonic::{Request, Response, Status};

pub mod email {
    tonic::include_proto!("email");
}

use email::email_service_server::EmailService as EmailServiceTrait;
use email::{
    ResendVerificationRequest, ResendVerificationResponse, SendVerificationRequest,
    SendVerificationResponse, VerifyCodeRequest, VerifyCodeResponse,
};

pub struct EmailServiceImpl {
    email_service: EmailService,
    verification_repo: VerificationRepository,
}

impl EmailServiceImpl {
    pub fn new(email_service: EmailService, verification_repo: VerificationRepository) -> Self {
        Self {
            email_service,
            verification_repo,
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

        Ok(Response::new(VerifyCodeResponse {
            success: true,
            message: "Email verified successfully".to_string(),
        }))
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
}
