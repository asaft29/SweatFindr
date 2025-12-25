use anyhow::Result;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;

const VERIFICATION_TTL_SECONDS: u64 = 900;
const REGISTRATION_TTL_SECONDS: u64 = 30 * 24 * 60 * 60;

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
}
