use tracing::warn;
use uuid::Uuid;

use super::super::{
    domain::social::{
        CommentMutation, CommentPresentation, NewPhotographComment, PhotographCommentBody,
        PhotographCommentResponse, VoteCounts,
    },
    error::PhotographyError,
};
use super::photography_service::PhotographyService;
use crate::features::accounts::domain::public_author::PublicAuthor;

impl PhotographyService {
    pub async fn present_comment(
        &self,
        presentation: CommentPresentation,
    ) -> PhotographCommentResponse {
        let flag = match presentation.author.country_code() {
            Some(code) => self.flags.country_flags(&[code]).await.remove(&code),
            None => None,
        };
        let badge = crate::features::blog::domain::post::UserBadgeInfo::from_public_author(
            &presentation.author,
            flag,
        );
        PhotographCommentResponse::from_comment_votestate_and_badge_info(
            presentation.comment,
            presentation.vote_state,
            presentation.author.public_user_id(),
            badge,
        )
    }
    pub async fn vote_photograph(
        &self,
        user_id: Uuid,
        photograph_id: Uuid,
        is_upvote: bool,
    ) -> Result<VoteCounts, PhotographyError> {
        self.repository
            .vote_photograph(user_id, photograph_id, is_upvote)
            .await
    }
    pub async fn rescind_photograph_vote(
        &self,
        user_id: Uuid,
        photograph_id: Uuid,
    ) -> Result<VoteCounts, PhotographyError> {
        self.repository
            .rescind_photograph_vote(user_id, photograph_id)
            .await
    }
    pub async fn vote_comment(
        &self,
        user_id: Uuid,
        comment_id: Uuid,
        is_upvote: bool,
    ) -> Result<VoteCounts, PhotographyError> {
        self.repository
            .vote_comment(user_id, comment_id, is_upvote)
            .await
    }
    pub async fn rescind_comment_vote(
        &self,
        user_id: Uuid,
        comment_id: Uuid,
    ) -> Result<VoteCounts, PhotographyError> {
        self.repository
            .rescind_comment_vote(user_id, comment_id)
            .await
    }
    pub async fn create_comment(
        &self,
        user_id: Uuid,
        photograph_id: Uuid,
        parent_id: Option<Uuid>,
        content: String,
    ) -> Result<CommentPresentation, PhotographyError> {
        let content =
            PhotographCommentBody::parse(content).map_err(|_| PhotographyError::InvalidInput)?;
        let mutation = self
            .repository
            .create_comment(NewPhotographComment {
                photograph_id,
                user_id,
                content: content.into_inner(),
                parent_comment_id: parent_id,
            })
            .await?;
        Ok(self.enrich_committed_comment(mutation).await)
    }
    pub async fn update_comment(
        &self,
        requester_id: Uuid,
        comment_id: Uuid,
        content: String,
    ) -> Result<CommentPresentation, PhotographyError> {
        let content =
            PhotographCommentBody::parse(content).map_err(|_| PhotographyError::InvalidInput)?;
        let mutation = self
            .repository
            .update_comment(requester_id, comment_id, content.into_inner())
            .await?;
        Ok(self.enrich_committed_comment(mutation).await)
    }
    pub async fn delete_comment(
        &self,
        requester_id: Uuid,
        comment_id: Uuid,
    ) -> Result<(), PhotographyError> {
        self.repository
            .delete_comment(requester_id, comment_id)
            .await
    }

    async fn enrich_committed_comment(&self, mutation: CommentMutation) -> CommentPresentation {
        let author_id = mutation.comment.user_id;
        let author = match self.repository.comment_author(author_id).await {
            Ok(author) => author,
            Err(error) => {
                warn!(%author_id, %error, "Committed photograph comment author enrichment failed; returning a deleted-author projection");
                PublicAuthor::deleted()
            }
        };
        CommentPresentation {
            comment: mutation.comment,
            vote_state: mutation.vote_state,
            author,
        }
    }
}
