//! Behavioral coverage for media persistence ordering and compensation.

use std::{collections::HashSet, path::PathBuf, sync::Arc};

use rust_be_template::util::media::{
    object_store::{
        MediaObjectStore, MediaObjectStoreFuture, ObjectLocation, ObjectStoreError,
        ObjectStoreOperation,
    },
    persistence::{MediaWriteError, PendingMediaObject, PersistedMedia, persist_media_objects},
};
use tokio::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    Upload(String),
    Persist,
    Delete(String),
}

struct RecordingStore {
    events: Arc<Mutex<Vec<Event>>>,
    failed_uploads: HashSet<String>,
    failed_deletes: HashSet<String>,
}

impl MediaObjectStore for RecordingStore {
    fn put_file<'a>(
        &'a self,
        location: ObjectLocation,
        _content_type: String,
        _source: PathBuf,
    ) -> MediaObjectStoreFuture<'a> {
        Box::pin(async move {
            self.events
                .lock()
                .await
                .push(Event::Upload(location.key().to_string()));
            if self.failed_uploads.contains(location.key()) {
                return Err(ObjectStoreError::new(
                    ObjectStoreOperation::Upload,
                    "injected upload failure",
                    true,
                ));
            }
            Ok(())
        })
    }

    fn delete<'a>(&'a self, location: ObjectLocation) -> MediaObjectStoreFuture<'a> {
        Box::pin(async move {
            self.events
                .lock()
                .await
                .push(Event::Delete(location.key().to_string()));
            if self.failed_deletes.contains(location.key()) {
                return Err(ObjectStoreError::new(
                    ObjectStoreOperation::Delete,
                    "injected delete failure",
                    true,
                ));
            }
            Ok(())
        })
    }
}

#[tokio::test]
async fn database_failure_deletes_new_object_after_persistence_attempt() -> Result<(), String> {
    let events = Arc::new(Mutex::new(Vec::new()));
    let store = RecordingStore {
        events: Arc::clone(&events),
        failed_uploads: HashSet::new(),
        failed_deletes: HashSet::new(),
    };
    let pending = [pending("new-profile.avif")];
    let result = persist_media_objects(&store, &pending, async {
        events.lock().await.push(Event::Persist);
        Err::<PersistedMedia<()>, _>("database rejected metadata")
    })
    .await;

    match result {
        Err(MediaWriteError::Persistence {
            source,
            compensation_failures,
        }) => {
            assert_eq!(source, "database rejected metadata");
            assert!(compensation_failures.is_empty());
        }
        _ => return Err("expected persistence failure".to_string()),
    }
    assert_eq!(
        *events.lock().await,
        vec![
            Event::Upload("new-profile.avif".to_string()),
            Event::Persist,
            Event::Delete("new-profile.avif".to_string()),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn superseded_object_is_deleted_only_after_database_commit() -> Result<(), String> {
    let events = Arc::new(Mutex::new(Vec::new()));
    let store = RecordingStore {
        events: Arc::clone(&events),
        failed_uploads: HashSet::new(),
        failed_deletes: HashSet::from(["old-profile.avif".to_string()]),
    };
    let pending = [pending("new-profile.avif")];
    let outcome = persist_media_objects(&store, &pending, async {
        events.lock().await.push(Event::Persist);
        Ok::<_, &'static str>(PersistedMedia::new(
            7_u8,
            vec![location("old-profile.avif")],
        ))
    })
    .await
    .map_err(|_| "expected successful database commit".to_string())?;

    assert_eq!(outcome.value, 7);
    assert_eq!(outcome.cleanup_failures.len(), 1);
    assert!(outcome.cleanup_failures[0].is_retryable());
    assert_eq!(
        *events.lock().await,
        vec![
            Event::Upload("new-profile.avif".to_string()),
            Event::Persist,
            Event::Delete("old-profile.avif".to_string()),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn later_upload_failure_compensates_prior_objects() -> Result<(), String> {
    let events = Arc::new(Mutex::new(Vec::new()));
    let store = RecordingStore {
        events: Arc::clone(&events),
        failed_uploads: HashSet::from(["thumbnail.avif".to_string()]),
        failed_deletes: HashSet::new(),
    };
    let pending = [pending("photograph.avif"), pending("thumbnail.avif")];
    let result = persist_media_objects(&store, &pending, async {
        events.lock().await.push(Event::Persist);
        Ok::<_, &'static str>(PersistedMedia::new((), Vec::new()))
    })
    .await;

    match result {
        Err(MediaWriteError::Upload {
            compensation_failures,
            ..
        }) if compensation_failures.is_empty() => {}
        _ => return Err("expected compensated upload failure".to_string()),
    }
    assert_eq!(
        *events.lock().await,
        vec![
            Event::Upload("photograph.avif".to_string()),
            Event::Upload("thumbnail.avif".to_string()),
            Event::Delete("thumbnail.avif".to_string()),
            Event::Delete("photograph.avif".to_string()),
        ]
    );
    Ok(())
}

fn location(key: &str) -> ObjectLocation {
    ObjectLocation::new("test-bucket", key)
}

fn pending(key: &str) -> PendingMediaObject {
    PendingMediaObject {
        location: location(key),
        content_type: "image/avif".to_string(),
        source: "unused-by-recording-store".into(),
    }
}
