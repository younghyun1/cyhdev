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
    // The permit moves into the blocking closure: if the awaiting request is
    // cancelled, the index work keeps running and must keep its slot.
    let result = tokio::task::spawn_blocking(move || {
        let result = task();
        drop(permit);
        result
    })
    .await?;
    result.map_err(BlogError::Search)
}
