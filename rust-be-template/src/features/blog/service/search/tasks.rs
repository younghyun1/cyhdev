use tokio::sync::Semaphore;

use super::super::super::error::BlogError;

const MAX_SEARCH_BLOCKING_JOBS: usize = 2;
static SEARCH_BLOCKING_JOBS: Semaphore = Semaphore::const_new(MAX_SEARCH_BLOCKING_JOBS);

pub async fn run_search_task<T, F>(task: F) -> Result<T, BlogError>
where
    T: Send + 'static,
    F: FnOnce() -> anyhow::Result<T> + Send + 'static,
{
    let permit = SEARCH_BLOCKING_JOBS
        .acquire()
        .await
        .map_err(|error| BlogError::Search(anyhow::anyhow!("search limiter closed: {error}")))?;
    let result = tokio::task::spawn_blocking(task).await?;
    drop(permit);
    result.map_err(BlogError::Search)
}
