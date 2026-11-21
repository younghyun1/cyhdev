//! The reader list in memory follows the store and fails closed.

use std::sync::Arc;

use kcna_archive_core::service::archive::Archive;
use uuid::Uuid;

use super::{
    access::ReaderStore, archive_service::KcnaArchiveService, test_store::MemoryReaderStore,
};
use crate::features::kcna_archive::error::ArchiveError;

async fn service(store: &Arc<MemoryReaderStore>) -> anyhow::Result<KcnaArchiveService> {
    let directory = tempfile::tempdir()?;
    // The directory has no snapshot; the reader list does not depend on one.
    Ok(KcnaArchiveService::start_with_store(
        directory.path().to_owned(),
        Arc::new(Archive::load_into_memory),
        Arc::clone(store) as _,
    )
    .await)
}

#[tokio::test]
async fn a_new_installation_admits_nobody_and_a_stored_list_is_applied_at_startup()
-> anyhow::Result<()> {
    let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
    let store = Arc::new(MemoryReaderStore::with_accounts([
        (first, "first"),
        (second, "second"),
    ]));
    let empty = service(&store).await?;
    assert!(!empty.has_readers().await);
    assert!(!empty.admits(first).await);
    assert!(empty.readers().await?.is_empty());

    // A service started later, as after a restart, applies what is stored.
    assert!(store.grant(first).await?);
    let restarted = service(&store).await?;
    assert!(restarted.has_readers().await);
    assert!(restarted.admits(first).await);
    assert!(!restarted.admits(second).await);
    Ok(())
}

#[tokio::test]
async fn changes_apply_once_stored_and_are_safe_to_repeat() -> anyhow::Result<()> {
    let (reader, other, actor) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let store = Arc::new(MemoryReaderStore::with_accounts([
        (reader, "reader"),
        (other, "other"),
    ]));
    let service = service(&store).await?;

    let listed = service.grant_reader(reader, actor).await?;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].user_name, "reader");
    assert!(service.admits(reader).await);
    assert!(!service.admits(other).await);
    assert_eq!(service.grant_reader(reader, actor).await?.len(), 1);

    // An account the store does not know as active is refused.
    assert!(matches!(
        service.grant_reader(Uuid::new_v4(), actor).await,
        Err(ArchiveError::ReaderNotFound)
    ));

    assert!(service.revoke_reader(reader, actor).await?.is_empty());
    assert!(!service.admits(reader).await);
    assert!(service.revoke_reader(reader, actor).await?.is_empty());
    Ok(())
}

#[tokio::test]
async fn a_failing_store_leaves_the_list_in_memory_as_it_was() -> anyhow::Result<()> {
    let (reader, other, actor) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let store = Arc::new(MemoryReaderStore::with_accounts([
        (reader, "reader"),
        (other, "other"),
    ]));
    let service = service(&store).await?;
    service.grant_reader(reader, actor).await?;

    store.fail(true);
    assert!(matches!(
        service.grant_reader(other, actor).await,
        Err(ArchiveError::Store)
    ));
    assert!(matches!(
        service.revoke_reader(reader, actor).await,
        Err(ArchiveError::Store)
    ));
    assert!(matches!(service.readers().await, Err(ArchiveError::Store)));
    assert!(service.admits(reader).await);
    assert!(!service.admits(other).await);
    Ok(())
}

#[tokio::test]
async fn a_failed_startup_read_admits_nobody_until_the_list_is_read_again() -> anyhow::Result<()> {
    let reader = Uuid::new_v4();
    let store = Arc::new(MemoryReaderStore::with_accounts([(reader, "reader")]));
    assert!(store.grant(reader).await?);
    store.fail(true);
    let service = service(&store).await?;
    assert!(!service.admits(reader).await);

    store.fail(false);
    assert_eq!(service.readers().await?.len(), 1);
    assert!(service.admits(reader).await);
    Ok(())
}
