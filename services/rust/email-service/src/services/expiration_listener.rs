use anyhow::Result;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tonic::Request;
use tracing::{error, info, warn};

pub mod auth {
    tonic::include_proto!("auth");
}

use auth::auth_service_client::AuthServiceClient;
use auth::DeleteUnverifiedUserRequest;

pub struct ExpirationListener {
    auth_service_url: String,
}

impl ExpirationListener {
    pub fn new(auth_service_url: String) -> Self {
        Self { auth_service_url }
    }

    pub async fn start(self: Arc<Self>, redis_url: String) -> Result<()> {
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
                        "Registration key expired, deleting user from database"
                    );

                    if let Err(e) = self.delete_expired_user(user_id).await {
                        error!(user_id = user_id, error = %e, "Failed to delete expired user");
                    }
                }
            }
        }

        Ok(())
    }

    async fn delete_expired_user(&self, user_id: i32) -> Result<()> {
        let mut client = AuthServiceClient::connect(self.auth_service_url.clone()).await?;

        match client
            .delete_unverified_user(Request::new(DeleteUnverifiedUserRequest { user_id }))
            .await
        {
            Ok(response) => {
                let res = response.into_inner();
                if res.success {
                    info!(
                        user_id = user_id,
                        message = res.message,
                        "Successfully deleted expired user"
                    );
                } else {
                    warn!(
                        user_id = user_id,
                        message = res.message,
                        "Could not delete expired user"
                    );
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!(
                "gRPC error deleting user {}: {}",
                user_id,
                e
            )),
        }
    }
}
