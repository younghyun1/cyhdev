use std::collections::HashMap;

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::schema::{post_tags, posts, tags};

use super::super::error::BlogError;
use super::blog_repository::BlogRepository;

pub const POST_SEARCH_REBUILD_PAGE_SIZE: i64 = 512;

pub struct SearchIndexPost {
    pub post_id: Uuid,
    pub title: String,
    pub tags: Vec<String>,
}

impl BlogRepository {
    pub async fn published_search_page(
        &self,
        after_post_id: Option<Uuid>,
    ) -> Result<Vec<SearchIndexPost>, BlogError> {
        let mut connection = self.connection().await?;
        let mut query = posts::table
            .filter(posts::post_is_published.eq(true))
            .select((posts::post_id, posts::post_title))
            .order(posts::post_id.asc())
            .into_boxed();
        if let Some(after_post_id) = after_post_id {
            query = query.filter(posts::post_id.gt(after_post_id));
        }
        let posts = query
            .limit(POST_SEARCH_REBUILD_PAGE_SIZE)
            .load::<(Uuid, String)>(&mut connection)
            .await?;
        if posts.is_empty() {
            return Ok(Vec::new());
        }
        let post_ids = posts
            .iter()
            .map(|(post_id, _)| *post_id)
            .collect::<Vec<_>>();
        let tag_rows = post_tags::table
            .inner_join(tags::table)
            .filter(post_tags::post_id.eq_any(post_ids))
            .select((post_tags::post_id, tags::tag_name))
            .load::<(Uuid, String)>(&mut connection)
            .await?;
        let mut tags_by_post = HashMap::<Uuid, Vec<String>>::new();
        for (post_id, tag) in tag_rows {
            tags_by_post.entry(post_id).or_default().push(tag);
        }
        Ok(posts
            .into_iter()
            .map(|(post_id, title)| SearchIndexPost {
                post_id,
                title,
                tags: tags_by_post.remove(&post_id).unwrap_or_default(),
            })
            .collect())
    }
}
