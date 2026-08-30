use std::collections::HashSet;

use tracing::warn;
use uuid::Uuid;

use crate::util::string::generate_slug::generate_slug;

use super::super::{
    domain::{
        cache::CachedPostInfo,
        comment::CommentResponse,
        post::{
            MAX_BLOG_POST_MARKDOWN_CHARS, MAX_BLOG_POST_TAG_CHARS, MAX_BLOG_POST_TAGS,
            MAX_BLOG_POST_TITLE_CHARS, PostInfo, PostLookup, ReadPostResult, SavePostCommand,
            SavePostInput, UserBadgeInfo,
        },
        vote::VoteState,
    },
    error::BlogError,
};
use super::blog_service::BlogService;

impl BlogService {
    pub async fn save_post(
        &self,
        input: SavePostInput,
    ) -> Result<super::super::domain::post::Post, BlogError> {
        let SavePostInput {
            actor_user_id,
            post_id,
            title,
            markdown,
            tags,
            published,
            owner_required,
        } = input;
        validate_post_text(&title, &markdown)?;
        let requested_tags = normalize_tags(tags)?;
        let source = markdown.clone();
        let rendered = tokio::task::spawn_blocking(move || {
            comrak::markdown_to_html(&source, &comrak::Options::default())
        })
        .await?;
        let mut post_use_case = match post_id {
            Some(post_id) => Some(self.lock_post_use_case(post_id).await),
            None => None,
        };
        let post = self
            .repository
            .save_post(SavePostCommand {
                post_id,
                actor_user_id,
                slug: generate_slug(&title),
                title,
                rendered_content: rendered,
                markdown_content: markdown,
                tags: requested_tags.clone(),
                published,
                owner_required,
            })
            .await?;
        if post_use_case.is_none() {
            post_use_case = Some(self.lock_post_use_case(post.post_id).await);
        }
        let cached =
            CachedPostInfo::from_post_info_with_tags(PostInfo::from(post.clone()), requested_tags);
        self.insert_cache(&cached).await;
        drop(post_use_case);
        Ok(post)
    }

    pub async fn delete_post(&self, requester_id: Uuid, post_id: Uuid) -> Result<(), BlogError> {
        let post_use_case = self.lock_post_use_case(post_id).await;
        self.repository.delete_post(requester_id, post_id).await?;
        self.delete_cache(post_id).await;
        drop(post_use_case);
        Ok(())
    }

    pub async fn read_post(
        &self,
        lookup: PostLookup,
        viewer_id: Option<Uuid>,
    ) -> Result<ReadPostResult, BlogError> {
        let post_id = match &lookup {
            PostLookup::Id(post_id) => *post_id,
            PostLookup::Slug(slug) => match self.cached_post_id_by_slug(slug).await {
                Some(post_id) => post_id,
                None => {
                    let post_id = self
                        .repository
                        .resolve_post_id(&lookup)
                        .await?
                        .ok_or(BlogError::PostNotFound)?;
                    self.cache_slug(slug, post_id).await;
                    post_id
                }
            },
        };
        let post_use_case = self.lock_post_use_case(post_id).await;
        let cached = self.cached_post(&post_id).await;
        let was_cached = cached.is_some();
        let mut post = self.repository.read_post(post_id, viewer_id).await?;
        // `post_content` is rendered at write time and remains authoritative.
        let post_tags = match cached {
            Some(post) => post.post_tags,
            None => self.repository.tags_for_post(post_id).await?,
        };
        let cached = CachedPostInfo::from_post_info_with_tags(
            PostInfo::from(post.clone()),
            post_tags.clone(),
        );
        if was_cached {
            self.insert_cache_without_search(&cached).await;
        } else {
            // A DB read-through also heals a missing search document while the
            // post stripe prevents an older read from overwriting a newer write.
            self.insert_cache(&cached).await;
        }
        drop(post_use_case);
        let comment_list = self.repository.comments_for_post(post_id).await?;
        if comment_list.truncated {
            tracing::warn!(
                post_id = %post_id,
                limit = super::super::repository::comments::MAX_COMPATIBILITY_POST_COMMENTS,
                "Blog read reached its fixed compatibility comment limit"
            );
        }
        let comments = comment_list.comments;
        let owner_user_id = post.user_id;
        let mut user_ids = comments
            .iter()
            .map(|comment| comment.user_id)
            .collect::<Vec<_>>();
        user_ids.push(owner_user_id);
        user_ids.sort_unstable();
        user_ids.dedup();
        let authors = self.repository.authors_by_ids(&user_ids).await?;
        let comment_ids = comments
            .iter()
            .map(|comment| comment.comment_id)
            .collect::<Vec<_>>();
        let comment_votes = self
            .repository
            .comment_vote_states(&comment_ids, viewer_id)
            .await?;
        let country_flags = self.country_flags_for_authors(&authors).await;
        let comment_responses = comments
            .into_iter()
            .map(|comment| {
                let vote = comment_votes
                    .get(&comment.comment_id)
                    .copied()
                    .unwrap_or(VoteState::DidNotVote);
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
                    comment, vote, public_id, badge,
                )
            })
            .collect();
        let post_badge = match authors.get(&owner_user_id) {
            Some(author) => {
                let flag = author
                    .country_code()
                    .and_then(|code| country_flags.get(&code).cloned());
                UserBadgeInfo::from_public_author(author, flag)
            }
            None => UserBadgeInfo::deleted(),
        };
        if authors
            .get(&owner_user_id)
            .is_none_or(|author| author.is_deleted())
        {
            post.user_id = Uuid::nil();
        }
        let vote_state = self.repository.post_vote_state(post_id, viewer_id).await?;
        let post_use_case = self.lock_post_use_case(post_id).await;
        match self.repository.increment_post_view(post_id).await {
            Ok(view_count) => {
                post.post_view_count = view_count;
                self.update_cached_views(post_id, view_count).await;
            }
            Err(error) => warn!(%post_id, %error,
                "Blog detail presentation succeeded but best-effort view persistence failed"),
        }
        drop(post_use_case);
        Ok(ReadPostResult {
            post,
            post_tags,
            comments: comment_responses,
            vote_state,
            user_badge_info: post_badge,
        })
    }
}

fn validate_post_text(title: &str, markdown: &str) -> Result<(), BlogError> {
    if title.trim().is_empty()
        || title.chars().count() > MAX_BLOG_POST_TITLE_CHARS
        || markdown.trim().is_empty()
        || markdown.chars().count() > MAX_BLOG_POST_MARKDOWN_CHARS
    {
        Err(BlogError::InvalidInput)
    } else {
        Ok(())
    }
}

fn normalize_tags(tags: Vec<String>) -> Result<Vec<String>, BlogError> {
    if tags.len() > MAX_BLOG_POST_TAGS
        || tags
            .iter()
            .any(|tag| tag.trim().chars().count() > MAX_BLOG_POST_TAG_CHARS)
    {
        return Err(BlogError::InvalidInput);
    }
    let mut seen = HashSet::new();
    Ok(tags
        .into_iter()
        .map(|tag| tag.trim().to_lowercase())
        .filter(|tag| !tag.is_empty())
        .filter(|tag| seen.insert(tag.clone()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{normalize_tags, validate_post_text};
    use crate::features::blog::domain::post::{MAX_BLOG_POST_TAGS, MAX_BLOG_POST_TITLE_CHARS};

    #[test]
    fn post_text_and_tags_are_bounded_before_rendering() {
        assert!(validate_post_text("", "body").is_err());
        assert!(validate_post_text(&"x".repeat(MAX_BLOG_POST_TITLE_CHARS + 1), "body").is_err());
        assert!(normalize_tags(vec!["tag".to_owned(); MAX_BLOG_POST_TAGS + 1]).is_err());
    }

    #[test]
    fn tags_are_normalized_and_deduplicated() {
        let tags = normalize_tags(vec![" Rust ".to_owned(), "rust".to_owned(), " ".to_owned()]);
        assert!(matches!(tags, Ok(tags) if tags == vec!["rust"]));
    }
}
