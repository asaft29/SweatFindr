use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct TokenBlacklist {
    tokens: Arc<RwLock<HashSet<String>>>,
    invalidated_users: Arc<RwLock<HashSet<i32>>>,
}

impl TokenBlacklist {
    pub async fn add(&self, token: String) {
        let mut tokens = self.tokens.write().await;
        tokens.insert(token);
    }

    pub async fn contains(&self, token: &str) -> bool {
        let tokens = self.tokens.read().await;
        tokens.contains(token)
    }

    pub async fn invalidate_user(&self, user_id: i32) {
        let mut users = self.invalidated_users.write().await;
        users.insert(user_id);
    }

    pub async fn is_user_invalidated(&self, user_id: i32) -> bool {
        let users = self.invalidated_users.read().await;
        users.contains(&user_id)
    }

    pub async fn clear_user_invalidation(&self, user_id: i32) {
        let mut users = self.invalidated_users.write().await;
        users.remove(&user_id);
    }
}

impl Default for TokenBlacklist {
    fn default() -> Self {
        TokenBlacklist {
            tokens: Arc::new(RwLock::new(HashSet::new())),
            invalidated_users: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}
