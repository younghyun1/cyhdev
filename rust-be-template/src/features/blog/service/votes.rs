use uuid::Uuid;

use super::blog_service::BlogService;
use super::super::{domain::vote::VoteCounts, error::BlogError};

impl BlogService {
    pub async fn vote_post(
        &self,
        user_id: Uuid,
        post_id: Uuid,
        is_upvote: bool,
    ) -> Result<VoteCounts, BlogError> {
        let post_use_case = self.lock_post_use_case(post_id).await;
        let counts = self.repository.vote_post(user_id, post_id, is_upvote).await?;
        self.update_cached_votes(post_id, counts.upvotes, counts.downvotes)
            .await;
        drop(post_use_case);
        Ok(counts)
    }

    pub async fn rescind_post_vote(
        &self,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<VoteCounts, BlogError> {
        let post_use_case = self.lock_post_use_case(post_id).await;
        let counts = self.repository.rescind_post_vote(user_id, post_id).await?;
        self.update_cached_votes(post_id, counts.upvotes, counts.downvotes)
            .await;
        drop(post_use_case);
        Ok(counts)
    }

    pub async fn vote_comment(
        &self,
        user_id: Uuid,
        comment_id: Uuid,
        is_upvote: bool,
    ) -> Result<VoteCounts, BlogError> {
        self.repository.vote_comment(user_id, comment_id, is_upvote).await
    }

    pub async fn rescind_comment_vote(
        &self,
        user_id: Uuid,
        comment_id: Uuid,
    ) -> Result<VoteCounts, BlogError> {
        self.repository.rescind_comment_vote(user_id, comment_id).await
    }
}
