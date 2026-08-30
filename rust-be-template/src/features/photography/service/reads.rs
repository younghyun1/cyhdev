use std::cmp::Reverse;
use uuid::Uuid;
use super::{photography_service::PhotographyService, super::{domain::{photograph::{PhotographPage, PresentedPhotographDetail}, social::PhotographCommentResponse}, error::PhotographyError}};

impl PhotographyService {
    pub async fn photographs(&self, page: i64, page_size: i64) -> Result<PhotographPage, PhotographyError> {
        if page < 1 || !(1..=100).contains(&page_size) { return Err(PhotographyError::InvalidInput); }
        self.repository.photograph_page(page, page_size).await
    }

    pub async fn photograph_detail(&self, photograph_id: Uuid, viewer: Option<Uuid>) -> Result<PresentedPhotographDetail, PhotographyError> {
        let mut detail = self.photograph_detail_with_view(photograph_id, viewer).await?;
        if detail.authors.get(&detail.owner_user_id).is_none_or(|author| author.is_deleted()) {
            detail.photograph.anonymize_deleted_owner();
        }
        detail.comments.sort_by_key(|(comment, _)| {
            Reverse(comment.photograph_comment_total_upvotes.saturating_sub(comment.photograph_comment_total_downvotes))
        });
        let mut country_codes = detail.authors.values().filter_map(|author| author.country_code()).collect::<Vec<_>>();
        country_codes.sort_unstable();
        country_codes.dedup();
        let flags = self.flags.country_flags(&country_codes).await;
        let comments = detail.comments.into_iter().map(|(comment, vote_state)| {
            let (public_id, badge) = match detail.authors.get(&comment.user_id) {
                Some(author) => {
                    let flag = author.country_code().and_then(|code| flags.get(&code).cloned());
                    (author.public_user_id(), crate::features::blog::domain::post::UserBadgeInfo::from_public_author(author, flag))
                }
                None => (Uuid::nil(), crate::features::blog::domain::post::UserBadgeInfo::deleted()),
            };
            PhotographCommentResponse::from_comment_votestate_and_badge_info(comment, vote_state, public_id, badge)
        }).collect();
        let author_badge = match detail.authors.get(&detail.owner_user_id) {
            Some(author) => {
                let flag = author.country_code().and_then(|code| flags.get(&code).cloned());
                crate::features::blog::domain::post::UserBadgeInfo::from_public_author(author, flag)
            }
            None => crate::features::blog::domain::post::UserBadgeInfo::deleted(),
        };
        Ok(PresentedPhotographDetail { photograph: detail.photograph, comments, vote_state: detail.vote_state, author_badge })
    }
}
