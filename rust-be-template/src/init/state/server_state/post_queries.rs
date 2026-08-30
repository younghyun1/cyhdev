use std::collections::HashMap;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, dsl::count_star};
use diesel_async::RunQueryDsl;
use tracing::error;
use uuid::Uuid;

use super::ServerState;
use crate::domain::blog::blog::{CachedPostInfo, PostInfo};
use crate::schema::{post_tags, posts, tags};

fn combine_posts(
    post_info: Vec<PostInfo>,
    tag_rows: Vec<(Uuid, String)>,
) -> Vec<CachedPostInfo> {
    let mut tags_by_post = HashMap::<Uuid, Vec<String>>::new();
    for (post_id, tag) in tag_rows {
        tags_by_post.entry(post_id).or_default().push(tag);
    }
    post_info
        .into_iter()
        .map(|post| {
            let post_tags = tags_by_post.remove(&post.post_id).unwrap_or_default();
            CachedPostInfo::from_post_info_with_tags(post, post_tags)
        })
        .collect()
}

impl ServerState {
    async fn get_posts_from_database(
        &self,
        page: usize,
        page_size: usize,
        include_unpublished: bool,
    ) -> anyhow::Result<(Vec<CachedPostInfo>, usize)> {
        let page_size = page_size.clamp(1, 100);
        let offset = page.saturating_sub(1).saturating_mul(page_size);
        let offset = i64::try_from(offset).unwrap_or(i64::MAX);
        let mut conn = self.get_conn().await?;

        let mut count_query = posts::table.select(count_star()).into_boxed();
        if !include_unpublished {
            count_query = count_query.filter(posts::post_is_published.eq(true));
        }
        let total_rows: i64 = count_query.first(&mut conn).await?;

        let mut page_query = posts::table.select(PostInfo::as_select()).into_boxed();
        if !include_unpublished {
            page_query = page_query.filter(posts::post_is_published.eq(true));
        }
        let post_info = page_query
            .order((posts::post_created_at.desc(), posts::post_id.desc()))
            .offset(offset)
            .limit(page_size as i64)
            .load::<PostInfo>(&mut conn)
            .await?;
        let post_ids = post_info.iter().map(|post| post.post_id).collect::<Vec<_>>();
        let tag_rows = if post_ids.is_empty() {
            Vec::new()
        } else {
            post_tags::table
                .inner_join(tags::table)
                .filter(post_tags::post_id.eq_any(&post_ids))
                .select((post_tags::post_id, tags::tag_name))
                .load::<(Uuid, String)>(&mut conn)
                .await?
        };
        drop(conn);

        let total_rows = usize::try_from(total_rows).unwrap_or_default();
        Ok((combine_posts(post_info, tag_rows), total_rows.div_ceil(page_size)))
    }

    pub async fn get_posts_from_cache(
        &self,
        page: usize,
        page_size: usize,
        include_unpublished: bool,
    ) -> (Vec<CachedPostInfo>, usize) {
        self.blog_cache_metrics.record_database_read_through();
        match self
            .get_posts_from_database(page, page_size, include_unpublished)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                error!(error = ?e, "PostgreSQL listing failed; using bounded metadata cache");
                self.get_posts_from_bounded_cache(page, page_size, include_unpublished)
                    .await
            }
        }
    }

    async fn posts_from_ids(&self, post_ids: Vec<Uuid>) -> Vec<CachedPostInfo> {
        let mut posts_by_id = HashMap::with_capacity(post_ids.len());
        let mut missing_ids = Vec::new();
        for post_id in &post_ids {
            match self.get_post_from_cache(post_id).await {
                Some(post) => {
                    posts_by_id.insert(*post_id, post);
                }
                None => missing_ids.push(*post_id),
            }
        }

        if !missing_ids.is_empty() {
            self.blog_cache_metrics.record_database_read_through();
            match self.load_posts_by_ids(&missing_ids).await {
                Ok(posts) => {
                    for post in posts {
                        posts_by_id.insert(post.post_id, post);
                    }
                }
                Err(e) => {
                    error!(error = ?e, missing_rows = missing_ids.len(), "Post search hydration failed");
                }
            }
        }

        post_ids
            .iter()
            .filter_map(|post_id| posts_by_id.get(post_id).cloned())
            .collect()
    }

    async fn load_posts_by_ids(&self, post_ids: &[Uuid]) -> anyhow::Result<Vec<CachedPostInfo>> {
        if post_ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut conn = self.get_conn().await?;
        let post_info = posts::table
            .filter(posts::post_id.eq_any(post_ids))
            .select(PostInfo::as_select())
            .load::<PostInfo>(&mut conn)
            .await?;
        let tag_rows = post_tags::table
            .inner_join(tags::table)
            .filter(post_tags::post_id.eq_any(post_ids))
            .select((post_tags::post_id, tags::tag_name))
            .load::<(Uuid, String)>(&mut conn)
            .await?;
        drop(conn);
        Ok(combine_posts(post_info, tag_rows))
    }

    async fn hydrate_search(
        &self,
        result: anyhow::Result<(Vec<Uuid>, usize)>,
        operation: &'static str,
    ) -> (Vec<CachedPostInfo>, usize) {
        match result {
            Ok((post_ids, total)) => (self.posts_from_ids(post_ids).await, total),
            Err(e) => {
                error!(error = ?e, operation, "Post search failed");
                (Vec::new(), 0)
            }
        }
    }

    pub async fn search_posts_by_title(
        &self,
        query: &str,
        offset: usize,
        limit: usize,
    ) -> (Vec<CachedPostInfo>, usize) {
        self.hydrate_search(
            self.search_index.search_by_title_paged(query, offset, limit),
            "title",
        ).await
    }

    pub async fn search_posts_by_title_and_tags(
        &self,
        query: &str,
        tags: &[String],
        offset: usize,
        limit: usize,
    ) -> (Vec<CachedPostInfo>, usize) {
        self.hydrate_search(
            self.search_index.search_by_title_and_tags_paged(query, tags, offset, limit),
            "title_and_tags",
        ).await
    }

    pub async fn search_posts_by_tags(
        &self,
        tags: &[String],
        offset: usize,
        limit: usize,
    ) -> (Vec<CachedPostInfo>, usize) {
        self.hydrate_search(
            self.search_index.search_by_tags_paged(tags, offset, limit),
            "tags",
        ).await
    }

    pub async fn search_posts_by_tag(
        &self,
        tag: &str,
        offset: usize,
        limit: usize,
    ) -> (Vec<CachedPostInfo>, usize) {
        self.hydrate_search(
            self.search_index.search_by_tag_paged(tag, offset, limit),
            "tag",
        ).await
    }
}
