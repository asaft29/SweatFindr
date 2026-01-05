use anyhow::Result;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

const VERIFICATION_TTL_SECONDS: u64 = 900;
const REGISTRATION_TTL_SECONDS: u64 = 30 * 24 * 60 * 60;
const PASSWORD_RESET_TTL_SECONDS: u64 = 900;
const RESET_TOKEN_TTL_SECONDS: u64 = 600;

pub struct VerificationRepository {
    redis: Arc<Mutex<redis::aio::ConnectionManager>>,
}

impl VerificationRepository {
    pub fn new(redis: redis::aio::ConnectionManager) -> Self {
        Self {
            redis: Arc::new(Mutex::new(redis)),
        }
    }

    fn verification_key(user_id: i32) -> String {
        format!("verification:{}", user_id)
    }

    fn registration_key(user_id: i32) -> String {
        format!("registration:{}", user_id)
    }

    fn password_reset_key(email: &str) -> String {
        format!("password_reset:{}", email)
    }

    fn reset_token_key(email: &str) -> String {
        format!("reset_token:{}", email)
    }

    pub async fn create_verification(&self, user_id: i32, verification_code: &str) -> Result<()> {
        let mut conn = self.redis.lock().await;

        conn.set_ex::<_, _, ()>(
            Self::verification_key(user_id),
            verification_code,
            VERIFICATION_TTL_SECONDS,
        )
        .await?;

        let timestamp = chrono::Utc::now().timestamp();
        conn.set_ex::<_, _, ()>(
            Self::registration_key(user_id),
            timestamp,
            REGISTRATION_TTL_SECONDS,
        )
        .await?;

        Ok(())
    }

    pub async fn verify_code(&self, user_id: i32, code: &str) -> Result<bool> {
        let mut conn = self.redis.lock().await;
        let stored_code: Option<String> = conn.get(Self::verification_key(user_id)).await?;

        match stored_code {
            Some(stored) if stored == code => {
                let _: () = conn.del(Self::verification_key(user_id)).await?;
                let _: () = conn.del(Self::registration_key(user_id)).await?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub async fn create_password_reset(&self, email: &str, reset_code: &str) -> Result<()> {
        let mut conn = self.redis.lock().await;

        conn.set_ex::<_, _, ()>(
            Self::password_reset_key(email),
            reset_code,
            PASSWORD_RESET_TTL_SECONDS,
        )
        .await?;

        Ok(())
    }

    pub async fn verify_password_reset_code(&self, email: &str, code: &str) -> Result<Option<String>> {
        let mut conn = self.redis.lock().await;
        let stored_code: Option<String> = conn.get(Self::password_reset_key(email)).await?;

        match stored_code {
            Some(stored) if stored == code => {
                let _: () = conn.del(Self::password_reset_key(email)).await?;

                let reset_token = Uuid::now_v7().to_string();
                conn.set_ex::<_, _, ()>(
                    Self::reset_token_key(email),
                    &reset_token,
                    RESET_TOKEN_TTL_SECONDS,
                )
                .await?;

                Ok(Some(reset_token))
            }
            _ => Ok(None),
        }
    }

    pub async fn validate_reset_token(&self, email: &str, token: &str) -> Result<bool> {
        let mut conn = self.redis.lock().await;
        let stored_token: Option<String> = conn.get(Self::reset_token_key(email)).await?;

        match stored_token {
            Some(stored) if stored == token => {
                let _: () = conn.del(Self::reset_token_key(email)).await?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
