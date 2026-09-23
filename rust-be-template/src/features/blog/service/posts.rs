use std::collections::HashSet;

use uuid::Uuid;

use crate::util::string::generate_slug::generate_slug;

use super::super::{
    domain::{
        cache::CachedPostInfo,
        post::{
            MAX_BLOG_POST_MARKDOWN_CHARS, MAX_BLOG_POST_TAG_CHARS, MAX_BLOG_POST_TAGS,
            MAX_BLOG_POST_TITLE_CHARS, PostInfo, SavePostCommand, SavePostInput,
        },
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
