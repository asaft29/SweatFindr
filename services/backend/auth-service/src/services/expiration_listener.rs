use crate::repository::UserRepository;
use crate::services::JwtService;
use anyhow::Result;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct ExpirationListener {
    user_repo: Arc<UserRepository>,
    client_service_url: String,
    jwt_service: Arc<JwtService>,
}

impl ExpirationListener {
    pub fn new(
        user_repo: Arc<UserRepository>,
        client_service_url: String,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            user_repo,
            client_service_url,
            jwt_service,
        }
    }

    async fn delete_client_from_client_service(&self, email: &str) {
        let Ok(token) = self.jwt_service.generate_token(0, "clients-service") else {
            return;
        };

        let _ = reqwest::Client::new()
            .post(format!("{}/api/internal/clients/delete-by-email", self.client_service_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({ "email": email }))
            .send()
            .await;
    }

    pub async fn start(&self, redis_url: String) -> Result<()> {
        info!("Starting Redis expiration listener...");

        let client = redis::Client::open(redis_url.as_str())?;
        let mut pubsub = client.get_async_pubsub().await?;
        pubsub.subscribe("__keyevent@0__:expired").await?;

        info!("Subscribed to Redis keyspace expiration events");

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            let payload: String = msg.get_payload()?;

            if let Some(user_id_str) = payload.strip_prefix("registration:") {
                if let Ok(user_id) = user_id_str.parse::<i32>() {
                    info!(user_id, "Registration expired, deleting unverified user");

                    if let Ok(Some(user)) = self.user_repo.find_by_id(user_id).await {
                        if user.email_verified {
                            continue;
                        }
                        self.delete_client_from_client_service(&user.email).await;
                    }

                    match self.user_repo.delete_unverified_user(user_id).await {
                        Ok(true) => info!(user_id, "Deleted expired user"),
                        Ok(false) => warn!(user_id, "User not found or already verified"),
                        Err(e) => error!(user_id, error = %e, "Failed to delete user"),
                    }
                }
            }
        }

        Ok(())
    }
}
