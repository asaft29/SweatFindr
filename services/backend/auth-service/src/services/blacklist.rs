use redis::aio::ConnectionManager;
use redis::AsyncCommands;

#[derive(Clone)]
pub struct TokenBlacklist {
    redis: ConnectionManager,
}

impl TokenBlacklist {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }

    pub async fn add(&self, token: String) {
        let mut conn = self.redis.clone();
        let key = format!("blacklist:token:{}", token);
        let _: Result<(), _> = conn.set_ex(&key, "1", 86400 * 7).await;
    }

    pub async fn contains(&self, token: &str) -> bool {
        let mut conn = self.redis.clone();
        let key = format!("blacklist:token:{}", token);
        conn.exists(&key).await.unwrap_or(false)
    }

    pub async fn invalidate_user(&self, user_id: i32) {
        let mut conn = self.redis.clone();
        let key = format!("invalidated:user:{}", user_id);
        let _: Result<(), _> = conn.set_ex(&key, "1", 86400 * 7).await;
    }

    pub async fn is_user_invalidated(&self, user_id: i32) -> bool {
        let mut conn = self.redis.clone();
        let key = format!("invalidated:user:{}", user_id);
        conn.exists(&key).await.unwrap_or(false)
    }

    pub async fn clear_user_invalidation(&self, user_id: i32) {
        let mut conn = self.redis.clone();
        let key = format!("invalidated:user:{}", user_id);
        let _: Result<(), _> = conn.del(&key).await;
    }
}
