use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::photography::{
        domain::photograph::{NewPhotograph, Photograph}, error::PhotographyError,
        repository::{photography_repository::PhotographyRepository, records::{NewPhotographRecord, PhotographRecord}},
    },
    persistence::{
        active_user::{ActiveUserWriteError, lock_active_superuser},
        media_cleanup::enqueue_media_cleanup,
        public_authors::load_deleted_user_ids,
    },
    schema::photographs,
    util::media::cleanup::{EnqueuedMediaCleanup, MediaCleanupRequest, REASON_DELETED_PHOTOGRAPH_IMAGE, REASON_DELETED_PHOTOGRAPH_THUMBNAIL},
};

pub(crate) struct RetiredPhotographs { pub(crate) cleanup: EnqueuedMediaCleanup, pub(crate) deleted_rows: usize }

impl PhotographyRepository {
    pub async fn insert_photograph(&self, user_id: Uuid, photograph: NewPhotograph) -> Result<Photograph, PhotographyError> {
        let mut connection = self.connection().await?;
        connection.transaction::<Photograph, PhotographyError, _>(async move |connection| {
            lock_superuser(connection, user_id).await?;
            let row = diesel::insert_into(photographs::table).values(NewPhotographRecord::from(photograph))
                .returning(PhotographRecord::as_returning()).get_result::<PhotographRecord>(&mut *connection).await?;
            Ok(row.into())
        }).await
    }

    pub(crate) async fn retire_photographs(&self, requester_id: Uuid, photograph_ids: &[Uuid]) -> Result<RetiredPhotographs, PhotographyError> {
        let mut connection = self.connection().await?;
        connection.transaction::<RetiredPhotographs, PhotographyError, _>(async move |connection| {
            lock_superuser(connection, requester_id).await?;
            let targets = photographs::table.filter(photographs::photograph_id.eq_any(photograph_ids))
                .select((photographs::photograph_id, photographs::photograph_link, photographs::photograph_thumbnail_link))
                .load::<(Uuid, String, String)>(&mut *connection).await?;
            let requests = targets.into_iter().flat_map(|(source_id, image_url, thumbnail_url)| [
                MediaCleanupRequest { original_url: image_url, reason: REASON_DELETED_PHOTOGRAPH_IMAGE, source_id },
                MediaCleanupRequest { original_url: thumbnail_url, reason: REASON_DELETED_PHOTOGRAPH_THUMBNAIL, source_id },
            ]).collect();
            let cleanup = enqueue_media_cleanup(connection, requests).await?;
            let deleted_rows = diesel::delete(photographs::table.filter(photographs::photograph_id.eq_any(photograph_ids)))
                .execute(&mut *connection).await?;
            Ok(RetiredPhotographs { cleanup, deleted_rows })
        }).await
    }

    pub async fn user_is_deleted(&self, user_id: Uuid) -> Result<bool, PhotographyError> {
        let mut connection = self.connection().await?;
        Ok(load_deleted_user_ids(&mut connection, &[user_id]).await?.contains(&user_id))
    }
}

async fn lock_superuser(connection: &mut diesel_async::AsyncPgConnection, user_id: Uuid) -> Result<(), PhotographyError> {
    lock_active_superuser(connection, user_id).await.map_err(|error| match error {
        ActiveUserWriteError::Inactive => PhotographyError::InactiveAccount,
        ActiveUserWriteError::Denied => PhotographyError::Forbidden,
        ActiveUserWriteError::TargetNotFound => PhotographyError::Forbidden,
        ActiveUserWriteError::Database(error) => PhotographyError::Query(error),
    })
}
