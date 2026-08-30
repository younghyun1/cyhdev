use tracing::{error, info};

use super::super::blog_service::BlogService;
use super::super::super::error::BlogError;

impl BlogService {
    pub async fn rebuild_search_index(&self) -> Result<usize, BlogError> {
        let mutation = self.search_index.lock_mutation().await;
        let index = self.search_index.clone();
        super::tasks::run_search_task(move || index.begin_rebuild()).await?;
        let rebuild = self.index_published_posts().await;
        match rebuild {
            Ok(indexed) => {
                let index = self.search_index.clone();
                if let Err(error_value) =
                    super::tasks::run_search_task(move || index.finish_rebuild()).await
                {
                    let index = self.search_index.clone();
                    let _ = super::tasks::run_search_task(move || index.abort_rebuild()).await;
                    return Err(error_value);
                }
                info!(posts_indexed = indexed, "Complete post search index rebuilt");
                drop(mutation);
                Ok(indexed)
            }
            Err(error_value) => {
                let index = self.search_index.clone();
                if let Err(rollback_error) =
                    super::tasks::run_search_task(move || index.abort_rebuild()).await
                {
                    error!(error = %rollback_error, "Failed to roll back post search rebuild");
                }
                drop(mutation);
                Err(error_value)
            }
        }
    }

    async fn index_published_posts(&self) -> Result<usize, BlogError> {
        let mut cursor = None;
        let mut indexed = 0usize;
        loop {
            let page = self.repository.published_search_page(cursor).await?;
            if page.is_empty() {
                break;
            }
            let page_count = page.len();
            let next_cursor = page.last().map(|post| post.post_id);
            let index = self.search_index.clone();
            super::tasks::run_search_task(move || {
                for post in page {
                    index.index_post(post.post_id, &post.title, &post.tags)?;
                }
                Ok(())
            })
            .await?;
            indexed += page_count;
            cursor = next_cursor;
            if page_count < 512 {
                break;
            }
        }
        Ok(indexed)
    }
}
