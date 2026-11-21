//! State machine tests: not installed, loading, ready, and failed.

use std::{
    path::Path,
    sync::{Arc, Condvar, Mutex, PoisonError},
    time::Duration,
};

use kcna_archive_core::{
    error::Error as CoreError,
    service::{archive::Archive, params::QueryParams},
};

use super::{
    archive_service::{KcnaArchiveService, SnapshotLoader},
    state::ServiceState,
    test_snapshot::{KOREAN_ARTICLE_ID, corrupt_manifest, write_snapshot},
};
use crate::features::kcna_archive::error::{ArchiveError, UnavailableReason};

/// Holds a loader until the test opens the gate.
#[derive(Default)]
pub(super) struct Gate {
    open: Mutex<bool>,
    wake: Condvar,
}

impl Gate {
    pub(super) fn release(&self) {
        let mut open = self.open.lock().unwrap_or_else(PoisonError::into_inner);
        *open = true;
        self.wake.notify_all();
    }

    pub(super) fn wait(&self) {
        let mut open = self.open.lock().unwrap_or_else(PoisonError::into_inner);
        while !*open {
            open = self.wake.wait(open).unwrap_or_else(PoisonError::into_inner);
        }
    }
}

/// A loader that blocks on `gate` and then reads the snapshot for real.
pub(super) fn gated_loader(gate: &Arc<Gate>) -> SnapshotLoader {
    let gate = Arc::clone(gate);
    Arc::new(move |directory: &Path| -> Result<Archive, CoreError> {
        gate.wait();
        Archive::load_into_memory(directory)
    })
}

pub(super) fn plain_loader() -> SnapshotLoader {
    Arc::new(Archive::load_into_memory)
}

/// Polls until the service reaches `state`; a bounded wait keeps a bug from hanging.
pub(super) async fn wait_for(
    service: &KcnaArchiveService,
    state: ServiceState,
) -> anyhow::Result<()> {
    for _ in 0..1000 {
        if service.status().await?.state == state {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    anyhow::bail!("service never reached {state:?}")
}

fn query(text: &str) -> QueryParams {
    QueryParams {
        q: text.to_owned(),
        ..QueryParams::default()
    }
}

#[tokio::test]
async fn a_missing_snapshot_is_not_installed() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    let service = KcnaArchiveService::start(directory.path().join("absent"), plain_loader()).await;
    wait_for(&service, ServiceState::NotInstalled).await?;
    let status = service.status().await?;
    assert!(status.snapshot.is_none() && !status.reloading);
    assert!(matches!(
        service.search(query("peace")).await,
        Err(ArchiveError::Unavailable(UnavailableReason::NotInstalled))
    ));
    // A directory without a manifest is also not installed.
    let empty = KcnaArchiveService::start(directory.path().to_owned(), plain_loader()).await;
    wait_for(&empty, ServiceState::NotInstalled).await
}

#[tokio::test]
async fn the_service_answers_loading_until_the_snapshot_is_read() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    write_snapshot(directory.path())?;
    let gate = Arc::new(Gate::default());
    let service = KcnaArchiveService::start(directory.path().to_owned(), gated_loader(&gate)).await;
    // Start returned while the loader is still blocked.
    wait_for(&service, ServiceState::Loading).await?;
    assert!(matches!(
        service.search(query("peace")).await,
        Err(ArchiveError::Unavailable(UnavailableReason::Loading))
    ));
    // The first load holds the gate, so a reload request conflicts.
    assert!(matches!(service.reload().await, Err(ArchiveError::Busy)));
    gate.release();
    wait_for(&service, ServiceState::Ready).await
}

#[tokio::test]
async fn a_ready_service_answers_every_use_case() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    write_snapshot(directory.path())?;
    let service = KcnaArchiveService::start(directory.path().to_owned(), plain_loader()).await;
    wait_for(&service, ServiceState::Ready).await?;

    let found = service.search(query("peace")).await?;
    assert_eq!(found.total, 2);
    let status = service.status().await?;
    assert_eq!(status.snapshot.map(|snapshot| snapshot.indexed), Some(3));
    let histogram = service.histogram(QueryParams::default()).await?;
    assert_eq!(
        histogram
            .buckets
            .iter()
            .map(|bucket| bucket.count)
            .sum::<u64>(),
        3
    );
    let trends = service
        .trends(QueryParams::default(), vec!["peace".to_owned()])
        .await?;
    assert_eq!(trends.series.len(), 1);
    let document = service
        .document(KOREAN_ARTICLE_ID.to_owned(), "핵".to_owned())
        .await?;
    assert!(
        document
            .paragraphs
            .iter()
            .flatten()
            .any(|segment| segment.hit)
    );
    assert!(matches!(
        service
            .document(
                "00000000-0000-0000-0000-0000000003e7".to_owned(),
                String::new()
            )
            .await,
        Err(ArchiveError::NotFound)
    ));
    assert!(matches!(
        service.document("nope".to_owned(), String::new()).await,
        Err(ArchiveError::BadRequest)
    ));
    let csv = service
        .export(
            query("peace"),
            kcna_archive_core::service::export::ExportFormat::Csv,
        )
        .await?;
    assert_eq!(csv.lines().count(), 3);
    Ok(())
}

#[tokio::test]
async fn a_snapshot_that_cannot_be_loaded_is_failed_and_a_reload_repairs_it() -> anyhow::Result<()>
{
    let directory = tempfile::tempdir()?;
    write_snapshot(directory.path())?;
    corrupt_manifest(directory.path())?;
    let service = KcnaArchiveService::start(directory.path().to_owned(), plain_loader()).await;
    wait_for(&service, ServiceState::Failed).await?;
    assert!(matches!(
        service.search(query("peace")).await,
        Err(ArchiveError::Unavailable(UnavailableReason::Failed))
    ));
    // Still broken: the reload fails and the state stays failed.
    assert!(matches!(
        service.reload().await,
        Err(ArchiveError::ReloadFailed)
    ));
    assert_eq!(service.status().await?.state, ServiceState::Failed);
    write_snapshot(directory.path())?;
    service.reload().await?;
    assert_eq!(service.status().await?.state, ServiceState::Ready);
    assert_eq!(service.search(query("peace")).await?.total, 2);
    Ok(())
}

#[tokio::test]
async fn reload_installs_a_snapshot_copied_after_startup() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    let service = KcnaArchiveService::start(directory.path().to_owned(), plain_loader()).await;
    wait_for(&service, ServiceState::NotInstalled).await?;
    assert!(matches!(
        service.reload().await,
        Err(ArchiveError::Unavailable(UnavailableReason::NotInstalled))
    ));
    write_snapshot(directory.path())?;
    service.reload().await?;
    assert_eq!(service.status().await?.state, ServiceState::Ready);
    Ok(())
}
