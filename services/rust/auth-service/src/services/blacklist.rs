use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct TokenBlacklist {
    tokens: Arc<RwLock<HashSet<String>>>,
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
}

impl Default for TokenBlacklist {
    fn default() -> Self {
        TokenBlacklist {
            tokens: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}
