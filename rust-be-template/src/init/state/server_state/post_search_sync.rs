use std::collections::HashMap;

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use tracing::{error, info};
use uuid::Uuid;

use super::ServerState;
use crate::schema::{post_tags, posts, tags};

const POST_SEARCH_REBUILD_PAGE_SIZE: i64 = 512;

impl ServerState {
    pub(super) async fn rebuild_post_search_index_from_db(&self) -> anyhow::Result<usize> {
        self.search_index.begin_rebuild()?;
        let rebuild = self.index_published_posts_from_db().await;
        match rebuild {
            Ok(indexed) => {
                if let Err(e) = self.search_index.finish_rebuild() {
                    let _ = self.search_index.abort_rebuild();
                    return Err(e);
                }
                info!(posts_indexed = indexed, "Complete post search index rebuilt");
                Ok(indexed)
            }
            Err(e) => {
                if let Err(rollback_error) = self.search_index.abort_rebuild() {
                    error!(error = ?rollback_error, "Failed to roll back post search rebuild");
                }
                Err(e)
            }
        }
    }

    async fn index_published_posts_from_db(&self) -> anyhow::Result<usize> {
        let mut conn = self.get_conn().await?;
        let mut cursor = None;
        let mut indexed = 0usize;

        loop {
            let mut query = posts::table
                .filter(posts::post_is_published.eq(true))
                .select((posts::post_id, posts::post_title))
                .into_boxed();
            if let Some(cursor) = cursor {
                query = query.filter(posts::post_id.gt(cursor));
            }
            let page = query
                .order(posts::post_id.asc())
                .limit(POST_SEARCH_REBUILD_PAGE_SIZE)
                .load::<(Uuid, String)>(&mut conn)
                .await?;
            if page.is_empty() {
                break;
            }

            let post_ids = page.iter().map(|(post_id, _)| *post_id).collect::<Vec<_>>();
            let tag_rows = post_tags::table
                .inner_join(tags::table)
                .filter(post_tags::post_id.eq_any(&post_ids))
                .select((post_tags::post_id, tags::tag_name))
                .load::<(Uuid, String)>(&mut conn)
                .await?;
            let mut tags_by_post = HashMap::<Uuid, Vec<String>>::new();
            for (post_id, tag) in tag_rows {
                tags_by_post.entry(post_id).or_default().push(tag);
            }

            for (post_id, title) in &page {
                let post_tags = tags_by_post.remove(post_id).unwrap_or_default();
                self.search_index.index_post(*post_id, title, &post_tags)?;
                indexed += 1;
            }
            cursor = page.last().map(|(post_id, _)| *post_id);
        }
        drop(conn);
        Ok(indexed)
    }
}
