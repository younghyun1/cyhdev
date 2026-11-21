//! PostgreSQL storage of the accounts allowed to read the archive.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, result::Error as DieselError};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl, pooled_connection::bb8::Pool};
use tracing::error;
use uuid::Uuid;

use crate::{
    features::kcna_archive::{
        domain::reader::{ArchiveReader, MAX_READERS},
        error::ArchiveError,
        service::access::ReaderStore,
    },
    schema::{kcna_archive_reader as reader, users},
};

type Row = (Uuid, String, String, DateTime<Utc>);

fn listed((user_id, user_name, user_email, granted_at): Row) -> ArchiveReader {
    ArchiveReader {
        user_id,
        user_name,
        user_email,
        granted_at,
    }
}

/// Logs a storage failure and reports it opaquely.
fn failed(action: &'static str, source: &dyn std::fmt::Display) -> ArchiveError {
    error!(error = %source, action, "KCNA archive reader storage failed");
    ArchiveError::Store
}

/// Reads and writes `kcna_archive_reader`.
pub struct ReaderRepository {
    pool: Pool<AsyncPgConnection>,
}

impl ReaderRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReaderStore for ReaderRepository {
    /// A row whose account was deleted since is skipped, not removed: such an
    /// account cannot sign in, and its retained row is the accounts feature's.
    async fn list(&self) -> Result<Vec<ArchiveReader>, ArchiveError> {
        let mut connection = match self.pool.get().await {
            Ok(connection) => connection,
            Err(source) => return Err(failed("connect", &source)),
        };
        let rows = reader::table
            .inner_join(users::table)
            .filter(users::user_deleted_at.is_null())
            .filter(users::user_hard_purged_at.is_null())
            .filter(users::user_is_system_actor.eq(false))
            .order((
                reader::kcna_archive_reader_granted_at.desc(),
                reader::kcna_archive_reader_user_id.asc(),
            ))
            .limit(MAX_READERS as i64)
            .select((
                reader::kcna_archive_reader_user_id,
                users::user_name,
                users::user_email,
                reader::kcna_archive_reader_granted_at,
            ))
            .load::<Row>(&mut connection)
            .await;
        match rows {
            Ok(rows) => Ok(rows.into_iter().map(listed).collect()),
            Err(source) => Err(failed("list", &source)),
        }
    }

    /// The share lock on the account row conflicts with the lock soft deletion
    /// takes, so an account cannot be deleted between the check and the insert.
    async fn grant(&self, user_id: Uuid) -> Result<bool, ArchiveError> {
        let mut connection = match self.pool.get().await {
            Ok(connection) => connection,
            Err(source) => return Err(failed("connect", &source)),
        };
        let granted = connection
            .transaction::<bool, DieselError, _>(async move |connection| {
                let active = users::table
                    .filter(users::user_id.eq(user_id))
                    .filter(users::user_deleted_at.is_null())
                    .filter(users::user_hard_purged_at.is_null())
                    .filter(users::user_is_system_actor.eq(false))
                    .select(users::user_id)
                    .for_share()
                    .first::<Uuid>(connection)
                    .await
                    .optional()?;
                if active.is_none() {
                    return Ok(false);
                }
                diesel::insert_into(reader::table)
                    .values(reader::kcna_archive_reader_user_id.eq(user_id))
                    .on_conflict_do_nothing()
                    .execute(connection)
                    .await?;
                Ok(true)
            })
            .await;
        match granted {
            Ok(granted) => Ok(granted),
            Err(source) => Err(failed("grant", &source)),
        }
    }

    async fn revoke(&self, user_id: Uuid) -> Result<(), ArchiveError> {
        let mut connection = match self.pool.get().await {
            Ok(connection) => connection,
            Err(source) => return Err(failed("connect", &source)),
        };
        let removed = diesel::delete(reader::table.find(user_id))
            .execute(&mut connection)
            .await;
        match removed {
            Ok(_) => Ok(()),
            Err(source) => Err(failed("revoke", &source)),
        }
    }
}
