use anyhow::Result;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;

const VERIFICATION_TTL_SECONDS: u64 = 900;

pub struct VerificationRepository {
    redis: Arc<Mutex<redis::aio::ConnectionManager>>,
}

impl VerificationRepository {
    pub fn new(redis: redis::aio::ConnectionManager) -> Self {
        Self {
            redis: Arc::new(Mutex::new(redis)),
        }
    }

    fn key(user_id: i32) -> String {
        format!("verification:{}", user_id)
    }

    pub async fn create_verification(&self, user_id: i32, verification_code: &str) -> Result<()> {
        let mut conn = self.redis.lock().await;
        conn.set_ex::<_, _, ()>(
            Self::key(user_id),
            verification_code,
            VERIFICATION_TTL_SECONDS,
        )
        .await?;
        Ok(())
    }

    pub async fn verify_code(&self, user_id: i32, code: &str) -> Result<bool> {
        let mut conn = self.redis.lock().await;
        let stored_code: Option<String> = conn.get(Self::key(user_id)).await?;

        match stored_code {
            Some(stored) if stored == code => {
                let _: () = conn.del(Self::key(user_id)).await?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
