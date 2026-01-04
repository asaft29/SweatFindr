use crate::repository::UserRepository;
use anyhow::Result;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct ExpirationListener {
    user_repo: Arc<UserRepository>,
}

impl ExpirationListener {
    pub fn new(user_repo: Arc<UserRepository>) -> Self {
        Self { user_repo }
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
                    info!(
                        user_id = user_id,
                        "Registration key expired, deleting unverified user"
                    );

                    match self.user_repo.delete_unverified_user(user_id).await {
                        Ok(true) => {
                            info!(user_id = user_id, "Successfully deleted expired user");
                        }
                        Ok(false) => {
                            warn!(
                                user_id = user_id,
                                "User not found or already verified, skipping deletion"
                            );
                        }
                        Err(e) => {
                            error!(user_id = user_id, error = %e, "Failed to delete expired user");
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
