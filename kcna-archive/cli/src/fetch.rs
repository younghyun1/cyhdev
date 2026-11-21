//! Bounded HTTP transport with per-collector pacing and retries.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::{Client, Response};
use tokio::{
    sync::{Mutex, Semaphore},
    time::{Instant, sleep, sleep_until},
};
use tracing::warn;
use url::Url;

use crate::error::{Error, Result};

const ATTEMPTS: usize = 6;

/// Largest accepted body. Archive pages are tens of kilobytes and the live
/// homepage about one megabyte, so anything near this size is not an article.
const MAX_RESPONSE_BYTES: usize = 32 * 1024 * 1024;

/// Shares one connection pool while bounding starts and in-flight requests.
pub struct Fetcher {
    client: Client,
    permits: Semaphore,
    next_start: Mutex<Instant>,
    start_interval: Duration,
}

impl Fetcher {
    /// Creates a transport tuned for slow, occasionally unavailable archive hosts.
    pub fn new(concurrency: usize, start_interval: Duration) -> Result<Self> {
        let client = Client::builder()
            .user_agent("kcna-research-archive/0.1 (+local academic corpus)")
            .cookie_store(true)
            .connect_timeout(Duration::from_secs(20))
            .timeout(Duration::from_secs(90))
            .pool_idle_timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            client,
            permits: Semaphore::new(concurrency),
            next_start: Mutex::new(Instant::now()),
            start_interval,
        })
    }

    /// Fetches a URL with bounded exponential backoff.
    pub async fn get(&self, url: &Url) -> Result<Vec<u8>> {
        self.request(|| self.client.get(url.clone()), url).await
    }

    /// Posts a KCNA pagination form with bounded exponential backoff.
    pub async fn post_form(&self, url: &Url, form: &[(&str, String)]) -> Result<Vec<u8>> {
        self.request(|| self.client.post(url.clone()).form(form), url)
            .await
    }

    async fn request<F>(&self, build: F, url: &Url) -> Result<Vec<u8>>
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        let mut delay = Duration::from_millis(500);
        for attempt in 1..=ATTEMPTS {
            let permit = self
                .permits
                .acquire()
                .await
                .map_err(|_| Error::Interrupted)?;
            self.wait_for_start().await;
            let outcome = attempt_once(build(), url).await;
            // Release the slot before backing off so a failing URL does not
            // hold concurrency away from healthy requests.
            drop(permit);
            match outcome {
                Ok(bytes) => return Ok(bytes),
                Err(error) if attempt < ATTEMPTS && error.is_retryable() => {
                    warn!(url = %url, error = %error, attempt, "Retrying remote request");
                }
                Err(error) => return Err(error),
            }
            sleep(delay + jitter(delay)).await;
            delay = (delay * 2).min(Duration::from_secs(12));
        }
        Err(Error::Incomplete { failures: 1 })
    }

    async fn wait_for_start(&self) {
        let mut next = self.next_start.lock().await;
        let now = Instant::now();
        if *next > now {
            sleep_until(*next).await;
        }
        *next = Instant::now() + self.start_interval;
    }
}

async fn attempt_once(request: reqwest::RequestBuilder, url: &Url) -> Result<Vec<u8>> {
    let response = request.send().await?.error_for_status()?;
    read_body(response, url).await
}

/// Reads the body incrementally so a truncated transfer is retried and an
/// oversized one is abandoned before it is buffered.
async fn read_body(mut response: Response, url: &Url) -> Result<Vec<u8>> {
    let too_large = || Error::ResponseTooLarge {
        url: url.to_string(),
        limit: MAX_RESPONSE_BYTES,
    };
    let declared = response
        .content_length()
        .and_then(|length| usize::try_from(length).ok());
    if declared.is_some_and(|length| length > MAX_RESPONSE_BYTES) {
        return Err(too_large());
    }
    let mut body = Vec::with_capacity(declared.unwrap_or(16 * 1024));
    while let Some(chunk) = response.chunk().await? {
        if body.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
            return Err(too_large());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// Spreads retries by up to half the backoff so requests that failed together
/// do not return together. The sub-second clock is enough entropy for pacing
/// and avoids a random-number dependency.
fn jitter(delay: Duration) -> Duration {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(elapsed) => u64::from(elapsed.subsec_nanos()),
        Err(_) => 0,
    };
    let span = u64::try_from(delay.as_millis() / 2)
        .unwrap_or(u64::MAX)
        .max(1);
    Duration::from_millis(nanos % span)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jitter_stays_below_half_the_delay() {
        let delay = Duration::from_millis(500);
        for _ in 0..64 {
            assert!(jitter(delay) < delay / 2);
        }
    }
}
