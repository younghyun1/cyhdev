use tracing::warn;
use uuid::Uuid;

use super::blog_service::BlogService;
use super::super::{
    domain::{comment::{BlogCommentBody, CommentResponse}, post::UserBadgeInfo, vote::VoteState},
    error::BlogError,
};

impl BlogService {
    pub async fn submit_comment(
        &self,
        user_id: Uuid,
        post_id: Uuid,
        parent_comment_id: Option<Uuid>,
        content: String,
    ) -> Result<CommentResponse, BlogError> {
        let content = BlogCommentBody::parse(content).map_err(|_| BlogError::InvalidInput)?.into_inner();
        let comment = self
            .repository
            .insert_comment(user_id, post_id, parent_comment_id, &content)
            .await?;
        Ok(self.present_committed_comment(comment).await)
    }

    pub async fn update_comment(
        &self,
        requester_id: Uuid,
        comment_id: Uuid,
        content: String,
    ) -> Result<CommentResponse, BlogError> {
        let content = BlogCommentBody::parse(content).map_err(|_| BlogError::InvalidInput)?.into_inner();
        let comment = self
            .repository
            .update_comment(requester_id, comment_id, &content)
            .await?;
        Ok(self.present_committed_comment(comment).await)
    }

    pub async fn delete_comment(
        &self,
        requester_id: Uuid,
        comment_id: Uuid,
    ) -> Result<(), BlogError> {
        self.repository.delete_comment(requester_id, comment_id).await
    }

    async fn present_committed_comment(
        &self,
        comment: super::super::domain::comment::Comment,
    ) -> CommentResponse {
        let authors = match self.repository.authors_by_ids(&[comment.user_id]).await {
            Ok(authors) => authors,
            Err(error) => {
                warn!(user_id = %comment.user_id, %error,
                    "Committed blog comment author enrichment failed; returning a deleted-author projection");
                std::collections::HashMap::new()
            }
        };
        let country_flags = self.country_flags_for_authors(&authors).await;
        let (public_id, badge) = match authors.get(&comment.user_id) {
            Some(author) => {
                let flag = author
                    .country_code()
                    .and_then(|code| country_flags.get(&code).cloned());
                (
                    author.public_user_id(),
                    UserBadgeInfo::from_public_author(author, flag),
                )
            }
            None => (Uuid::nil(), UserBadgeInfo::deleted()),
        };
        CommentResponse::from_comment_votestate_and_badge_info(
            comment,
            VoteState::DidNotVote,
            public_id,
            badge,
        )
    }
}
