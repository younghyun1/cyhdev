//! Injectable object-store boundary for media persistence.

use std::{future::Future, path::PathBuf, pin::Pin};

use aws_sdk_s3::primitives::ByteStream;

/// An object address independent of a concrete object-store client.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ObjectLocation {
    bucket: String,
    key: String,
}

impl ObjectLocation {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            key: key.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn public_s3_url(&self, region: &str) -> String {
        format!(
            "https://{}.s3.{}.amazonaws.com/{}",
            self.bucket, region, self.key
        )
    }

    /// Parses a URL produced by [`Self::public_s3_url`] for the expected bucket.
    pub fn from_public_s3_url(expected_bucket: &str, value: &str) -> Option<Self> {
        let url = match reqwest::Url::parse(value) {
            Ok(url) => url,
            Err(_) => return None,
        };
        let host = url.host_str()?;
        let regional_prefix = format!("{expected_bucket}.s3.");
        let global_host = format!("{expected_bucket}.s3.amazonaws.com");
        let valid_host = host == global_host
            || (host.starts_with(&regional_prefix) && host.ends_with(".amazonaws.com"));
        let key = url.path().trim_start_matches('/');
        if !valid_host || key.is_empty() {
            return None;
        }
        Some(Self::new(expected_bucket, key))
    }
}

/// Object-store operation associated with a failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectStoreOperation {
    ReadSource,
    Upload,
    Delete,
}

/// Erased object-store failure with retry classification.
#[derive(Debug, thiserror::Error)]
#[error("{operation:?} failed: {message}")]
pub struct ObjectStoreError {
    operation: ObjectStoreOperation,
    message: String,
    retryable: bool,
}

impl ObjectStoreError {
    pub fn new(
        operation: ObjectStoreOperation,
        message: impl Into<String>,
        retryable: bool,
    ) -> Self {
        Self {
            operation,
            message: message.into(),
            retryable,
        }
    }

    pub fn operation(&self) -> ObjectStoreOperation {
        self.operation
    }

    pub fn is_retryable(&self) -> bool {
        self.retryable
    }
}

/// Minimal media object-store contract used by compensation workflows.
pub type MediaObjectStoreFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), ObjectStoreError>> + Send + 'a>>;

pub trait MediaObjectStore: Send + Sync {
    fn put_file<'a>(
        &'a self,
        location: ObjectLocation,
        content_type: String,
        source: PathBuf,
    ) -> MediaObjectStoreFuture<'a>;

    fn delete<'a>(&'a self, location: ObjectLocation) -> MediaObjectStoreFuture<'a>;
}

/// AWS S3 implementation of [`MediaObjectStore`].
#[derive(Clone)]
pub struct S3MediaObjectStore {
    client: aws_sdk_s3::Client,
}

impl S3MediaObjectStore {
    pub fn from_config(config: &aws_config::SdkConfig) -> Self {
        Self {
            client: aws_sdk_s3::Client::new(config),
        }
    }
}

impl MediaObjectStore for S3MediaObjectStore {
    fn put_file<'a>(
        &'a self,
        location: ObjectLocation,
        content_type: String,
        source: PathBuf,
    ) -> MediaObjectStoreFuture<'a> {
        Box::pin(async move {
            let body = ByteStream::from_path(source).await.map_err(|error| {
                ObjectStoreError::new(ObjectStoreOperation::ReadSource, error.to_string(), false)
            })?;
            self.client
                .put_object()
                .bucket(location.bucket())
                .key(location.key())
                .content_type(content_type)
                .body(body)
                .send()
                .await
                .map_err(|error| {
                    ObjectStoreError::new(ObjectStoreOperation::Upload, error.to_string(), true)
                })?;
            Ok(())
        })
    }

    fn delete<'a>(&'a self, location: ObjectLocation) -> MediaObjectStoreFuture<'a> {
        Box::pin(async move {
            self.client
                .delete_object()
                .bucket(location.bucket())
                .key(location.key())
                .send()
                .await
                .map_err(|error| {
                    // Delete is idempotent, so every remote failure is safe to retry.
                    ObjectStoreError::new(ObjectStoreOperation::Delete, error.to_string(), true)
                })?;
            Ok(())
        })
    }
}
