//! Ordered object-store/database persistence with explicit compensation.

use std::{collections::HashSet, future::Future, path::PathBuf, pin::Pin};

use super::object_store::{
    MediaObjectStore, ObjectLocation, ObjectStoreError, ObjectStoreOperation,
};

/// A processed file waiting to be uploaded.
#[derive(Clone, Debug)]
pub struct PendingMediaObject {
    pub location: ObjectLocation,
    pub content_type: String,
    pub source: PathBuf,
}

/// Database commit result plus objects superseded by the new metadata.
#[derive(Debug)]
pub struct PersistedMedia<T> {
    pub value: T,
    pub superseded: Vec<ObjectLocation>,
}

impl<T> PersistedMedia<T> {
    pub fn new(value: T, superseded: Vec<ObjectLocation>) -> Self {
        Self { value, superseded }
    }
}

/// A failed cleanup which callers may enqueue for retry.
#[derive(Debug)]
pub struct CleanupFailure {
    pub location: ObjectLocation,
    pub error: ObjectStoreError,
}

impl CleanupFailure {
    pub fn is_retryable(&self) -> bool {
        self.error.is_retryable()
    }
}

/// Successful database commit, including any deferred cleanup work.
#[derive(Debug)]
pub struct MediaWriteSuccess<T> {
    pub value: T,
    pub cleanup_failures: Vec<CleanupFailure>,
}

/// Failure before the database commit, including incomplete compensation.
#[derive(Debug)]
pub enum MediaWriteError<E> {
    Upload {
        source: ObjectStoreError,
        compensation_failures: Vec<CleanupFailure>,
    },
    Persistence {
        source: E,
        compensation_failures: Vec<CleanupFailure>,
    },
}

/// Boxed ordered media-write operation returned across the object-store boundary.
pub type MediaWriteFuture<'a, T, E> = Pin<
    Box<dyn Future<Output = Result<MediaWriteSuccess<T>, MediaWriteError<E>>> + Send + 'a>,
>;

/// Uploads new objects, commits their metadata, then removes superseded objects.
///
/// A failed upload or database commit deletes every newly uploaded object. Old
/// objects are never deleted until the database commit has made the new media
/// authoritative. Cleanup failures remain explicit and are safe to retry.
pub fn persist_media_objects<'a, T, E, PersistFuture>(
    store: &'a dyn MediaObjectStore,
    pending: &'a [PendingMediaObject],
    persist: PersistFuture,
) -> MediaWriteFuture<'a, T, E>
where
    T: Send + 'a,
    E: Send + 'a,
    PersistFuture: Future<Output = Result<PersistedMedia<T>, E>> + Send + 'a,
{
    Box::pin(async move {
        let mut uploaded = Vec::with_capacity(pending.len());
        for object in pending {
            if let Err(source) = store
                .put_file(
                    object.location.clone(),
                    object.content_type.clone(),
                    object.source.clone(),
                )
                .await
            {
                // A retryable send failure is ambiguous: S3 may have committed the
                // PUT before the response was lost. Delete the attempted key too.
                if source.operation() == ObjectStoreOperation::Upload {
                    uploaded.push(object.location.clone());
                }
                let compensation_failures =
                    cleanup_objects(store, uploaded.iter().rev().cloned().collect()).await;
                return Err(MediaWriteError::Upload {
                    source,
                    compensation_failures,
                });
            }
            uploaded.push(object.location.clone());
        }

        let persisted = match persist.await {
            Ok(persisted) => persisted,
            Err(source) => {
                let compensation_failures =
                    cleanup_objects(store, uploaded.iter().rev().cloned().collect()).await;
                return Err(MediaWriteError::Persistence {
                    source,
                    compensation_failures,
                });
            }
        };

        let uploaded_set: HashSet<&ObjectLocation> = uploaded.iter().collect();
        let superseded = persisted
            .superseded
            .iter()
            .filter(|location| !uploaded_set.contains(location))
            .cloned()
            .collect();
        let cleanup_failures = cleanup_objects(store, superseded).await;
        Ok(MediaWriteSuccess {
            value: persisted.value,
            cleanup_failures,
        })
    })
}

async fn cleanup_objects(
    store: &dyn MediaObjectStore,
    locations: Vec<ObjectLocation>,
) -> Vec<CleanupFailure> {
    let mut failures = Vec::new();
    let mut visited = HashSet::new();
    for location in locations {
        if !visited.insert(location.clone()) {
            continue;
        }
        if let Err(error) = store.delete(location.clone()).await {
            failures.push(CleanupFailure { location, error });
        }
    }
    failures
}
