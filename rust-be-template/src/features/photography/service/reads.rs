use std::collections::HashMap;

use uuid::Uuid;

use super::{
    super::{
        domain::{
            photograph::{PhotographPage, PresentedPhotographDetail},
            social::{PhotographCommentPage, PhotographCommentPageData, PhotographCommentResponse},
        },
        error::PhotographyError,
    },
    photography_service::PhotographyService,
};
use crate::features::blog::domain::{comment_page::CommentPageRequest, post::UserBadgeInfo};

impl PhotographyService {
    pub async fn photographs(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<PhotographPage, PhotographyError> {
        if page < 1 || !(1..=100).contains(&page_size) {
            return Err(PhotographyError::InvalidInput);
        }
        self.repository.photograph_page(page, page_size).await
    }

    /// Reads one photograph with its first oldest-first comment page. Clients
    /// order and thread the accumulated pages themselves.
    pub async fn photograph_detail(
        &self,
        photograph_id: Uuid,
        viewer: Option<Uuid>,
    ) -> Result<PresentedPhotographDetail, PhotographyError> {
        let mut detail = self
            .photograph_detail_with_view(photograph_id, viewer)
            .await?;
        if detail
            .comments
            .authors
            .get(&detail.owner_user_id)
            .is_none_or(|author| author.is_deleted())
        {
            detail.photograph.anonymize_deleted_owner();
        }
        let flags = self.author_flags(&detail.comments).await;
        let (_, author_badge) =
            UserBadgeInfo::resolve(&detail.comments.authors, &flags, detail.owner_user_id);
        let comments_next_cursor = detail.comments.next_cursor;
        Ok(PresentedPhotographDetail {
            photograph: detail.photograph,
            comments: present_comments(detail.comments, &flags),
            comments_next_cursor,
            vote_state: detail.vote_state,
            author_badge,
        })
    }

    /// Reads a later comment page; comment reads never count a view.
    pub async fn comment_page(
        &self,
        photograph_id: Uuid,
        viewer: Option<Uuid>,
        request: CommentPageRequest,
    ) -> Result<PhotographCommentPage, PhotographyError> {
        let page = self
            .repository
            .comment_page(photograph_id, viewer, request)
            .await?;
        let flags = self.author_flags(&page).await;
        let next_cursor = page.next_cursor;
        Ok(PhotographCommentPage {
            comments: present_comments(page, &flags),
            next_cursor,
        })
    }

    async fn author_flags(&self, page: &PhotographCommentPageData) -> HashMap<i32, String> {
        let mut country_codes = page
            .authors
            .values()
            .filter_map(|author| author.country_code())
            .collect::<Vec<_>>();
        country_codes.sort_unstable();
        country_codes.dedup();
        self.flags.country_flags(&country_codes).await
    }
}

fn present_comments(
    page: PhotographCommentPageData,
    flags: &HashMap<i32, String>,
) -> Vec<PhotographCommentResponse> {
    page.comments
        .into_iter()
        .map(|(comment, vote_state)| {
            let (public_id, badge) = UserBadgeInfo::resolve(&page.authors, flags, comment.user_id);
            PhotographCommentResponse::from_comment_votestate_and_badge_info(
                comment, vote_state, public_id, badge,
            )
        })
        .collect()
}
