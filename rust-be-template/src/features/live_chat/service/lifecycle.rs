use uuid::Uuid;

use super::cache::LiveChatCache;

#[async_trait::async_trait]
pub trait LiveChatAccountLifecyclePort: Send + Sync {
    async fn anonymize_deleted_account(&self, user_id: Uuid);
}

#[async_trait::async_trait]
impl LiveChatAccountLifecyclePort for LiveChatCache {
    async fn anonymize_deleted_account(&self, user_id: Uuid) {
        self.anonymize_deleted_user(user_id).await;
    }
}
