use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Cache for user_id -> email mapping
/// In a production system, this would query the auth service or database
pub struct UserEmailCache {
    cache: Arc<RwLock<HashMap<i32, String>>>,
}

impl UserEmailCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get email for a user_id
    /// For now, we'll use the client repo to get email by looking up the client
    /// In a real system, you'd call the auth service
    pub async fn get_email(&self, user_id: i32) -> Option<String> {
        let cache = self.cache.read().await;
        cache.get(&user_id).cloned()
    }

    /// Cache an email for a user_id
    pub async fn cache_email(&self, user_id: i32, email: String) {
        let mut cache = self.cache.write().await;
        cache.insert(user_id, email);
    }
}

impl Default for UserEmailCache {
    fn default() -> Self {
        Self::new()
    }
}
