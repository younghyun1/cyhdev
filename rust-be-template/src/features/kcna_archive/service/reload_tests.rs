//! Reload behavior while a snapshot is serving.

use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use kcna_archive_core::service::{archive::Archive, params::QueryParams};

use super::{
    archive_service::{KcnaArchiveService, SnapshotLoader},
    state::ServiceState,
    test_snapshot::{corrupt_manifest, write_snapshot},
    tests::{Gate, plain_loader, wait_for},
};
use crate::features::kcna_archive::error::ArchiveError;

fn query(text: &str) -> QueryParams {
    QueryParams {
        q: text.to_owned(),
        ..QueryParams::default()
    }
}

/// A loader that reads at once the first time and blocks on `gate` afterwards,
/// so the startup load completes and the first reload can be held open.
fn blocks_after_first_load(gate: &Arc<Gate>) -> SnapshotLoader {
    let gate = Arc::clone(gate);
    let calls = Arc::new(AtomicUsize::new(0));
    Arc::new(move |directory: &Path| {
        if calls.fetch_add(1, Ordering::SeqCst) > 0 {
            gate.wait();
        }
        Archive::load_into_memory(directory)
    })
}

#[tokio::test]
async fn a_second_reload_conflicts_and_the_old_snapshot_keeps_serving() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    write_snapshot(directory.path())?;
    let gate = Arc::new(Gate::default());
    let service =
        KcnaArchiveService::start(directory.path().to_owned(), blocks_after_first_load(&gate))
            .await;
    wait_for(&service, ServiceState::Ready).await?;

    let reload = tokio::spawn({
        let service = service.clone();
        async move { service.reload().await }
    });
    // Wait until the reload holds the gate inside the blocked loader.
    for _ in 0..1000 {
        if service.status().await?.reloading {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(service.status().await?.reloading);
    assert!(matches!(service.reload().await, Err(ArchiveError::Busy)));
    // The old snapshot keeps answering while the reload is blocked.
    assert_eq!(service.search(query("peace")).await?.total, 2);

    gate.release();
    reload.await??;
    let status = service.status().await?;
    assert_eq!(status.state, ServiceState::Ready);
    assert!(!status.reloading && !status.last_reload_failed);
    Ok(())
}

#[tokio::test]
async fn a_failed_reload_keeps_the_old_snapshot_and_reports_it() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    write_snapshot(directory.path())?;
    let service = KcnaArchiveService::start(directory.path().to_owned(), plain_loader()).await;
    wait_for(&service, ServiceState::Ready).await?;

    corrupt_manifest(directory.path())?;
    assert!(matches!(
        service.reload().await,
        Err(ArchiveError::ReloadFailed)
    ));
    let status = service.status().await?;
    assert_eq!(status.state, ServiceState::Ready);
    assert!(status.last_reload_failed);
    assert_eq!(service.search(query("peace")).await?.total, 2);

    write_snapshot(directory.path())?;
    service.reload().await?;
    assert!(!service.status().await?.last_reload_failed);
    Ok(())
}
