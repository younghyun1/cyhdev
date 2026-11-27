use diesel::{
    ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper,
    sql_types::{Array, BigInt, Uuid as SqlUuid},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use tracing::warn;
use uuid::Uuid;

use crate::{
    features::blog::domain::{comment_page::CommentPageRequest, vote::VoteState},
    features::photography::{
        domain::photograph::{Photograph, PhotographDetail, PhotographPage},
        error::PhotographyError,
        repository::{
            comment_reads::{finish_comment_page, load_comment_rows},
            enums::DbPhotographContext,
            photography_repository::PhotographyRepository,
            records::PhotographRecord,
        },
    },
    persistence::public_authors::load_deleted_user_ids,
    schema::{photograph_votes, photographs},
};

const VIEW_DELTA_CHUNK_SIZE: usize = 256;
const APPLY_VIEW_DELTAS_SQL: &str = "\
UPDATE photographs AS photograph \
SET photograph_view_count = photograph.photograph_view_count + view_delta.delta \
FROM unnest($1::uuid[], $2::bigint[]) AS view_delta(photograph_id, delta) \
WHERE photograph.photograph_id = view_delta.photograph_id";

impl PhotographyRepository {
    pub async fn apply_view_deltas(
        &self,
        pending: &[(Uuid, i64)],
    ) -> Result<Vec<(Uuid, i64)>, PhotographyError> {
        let mut connection = self.connection().await?;
        let mut applied = Vec::with_capacity(pending.len());
        for chunk in pending.chunks(VIEW_DELTA_CHUNK_SIZE) {
            let photograph_ids = chunk
                .iter()
                .map(|(photograph_id, _)| *photograph_id)
                .collect::<Vec<_>>();
            let deltas = chunk.iter().map(|(_, delta)| *delta).collect::<Vec<_>>();
            let result = diesel::sql_query(APPLY_VIEW_DELTAS_SQL)
                .bind::<Array<SqlUuid>, _>(photograph_ids)
                .bind::<Array<BigInt>, _>(deltas)
                .execute(&mut connection)
                .await;
            match result {
                Ok(_) => applied.extend_from_slice(chunk),
                Err(error) => {
                    warn!(chunk_size = chunk.len(), %error, "Photograph view-delta chunk remains queued")
                }
            }
        }
        Ok(applied)
    }

    pub async fn increment_view(&self, photograph_id: Uuid) -> Result<(), PhotographyError> {
        let mut connection = self.connection().await?;
        let affected =
            diesel::update(photographs::table.filter(photographs::photograph_id.eq(photograph_id)))
                .set(
                    photographs::photograph_view_count
                        .eq(photographs::photograph_view_count + 1_i64),
                )
                .execute(&mut connection)
                .await?;
        if affected == 0 {
            Err(PhotographyError::PhotographNotFound)
        } else {
            Ok(())
        }
    }
    pub async fn photograph_page(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<PhotographPage, PhotographyError> {
        let connection = self.connection().await?;
        let connection: &AsyncPgConnection = &connection;
        // The total and the page are independent statements; joining them
        // sends both before reading either result.
        let total = photographs::table
            .filter(photographs::photograph_context.eq(DbPhotographContext::Photography))
            .count()
            .get_result::<i64>(&mut &*connection);
        let page_rows = photographs::table
            .filter(photographs::photograph_context.eq(DbPhotographContext::Photography))
            .order((
                photographs::photograph_shot_at.desc(),
                photographs::photograph_id.desc(),
            ))
            .offset((page - 1).saturating_mul(page_size))
            .limit(page_size)
            .select(PhotographRecord::as_select())
            .load::<PhotographRecord>(&mut &*connection);
        let (total_items, records) = tokio::try_join!(total, page_rows)?;
        let owner_ids = records
            .iter()
            .map(|record| record.clone_author_id())
            .collect::<Vec<_>>();
        let deleted = load_deleted_user_ids(connection, &owner_ids).await?;
        let mut items = records
            .into_iter()
            .map(Photograph::from)
            .collect::<Vec<_>>();
        for photograph in &mut items {
            if deleted.contains(&photograph.user_id) {
                photograph.anonymize_deleted_owner();
            }
        }
        Ok(PhotographPage {
            items,
            page,
            page_size,
            total_items,
        })
    }

    /// Reads a photograph, its first comment page, and the viewer's vote in
    /// two round trips: the three reads keyed by the photograph first, then
    /// the author batch and comment votes that need their rows.
    pub async fn photograph_detail(
        &self,
        photograph_id: Uuid,
        viewer: Option<Uuid>,
    ) -> Result<PhotographDetail, PhotographyError> {
        let connection = self.connection().await?;
        let connection: &AsyncPgConnection = &connection;
        let photograph = async {
            photographs::table
                .filter(photographs::photograph_id.eq(photograph_id))
                .select(PhotographRecord::as_select())
                .first::<PhotographRecord>(&mut &*connection)
                .await
                .optional()
        };
        let viewer_vote = async {
            match viewer {
                Some(user_id) => photograph_votes::table
                    .filter(photograph_votes::photograph_id.eq(photograph_id))
                    .filter(photograph_votes::user_id.eq(user_id))
                    .select(photograph_votes::is_upvote)
                    .first::<bool>(&mut &*connection)
                    .await
                    .optional(),
                None => Ok(None),
            }
        };
        // A missing photograph still runs the comment and vote probes; their
        // results are dropped with the error below.
        let (record, comment_rows, viewer_vote) = tokio::try_join!(
            photograph,
            load_comment_rows(connection, photograph_id, CommentPageRequest::first_page()),
            viewer_vote
        )?;
        let record = record.ok_or(PhotographyError::PhotographNotFound)?;
        let owner_user_id = record.clone_author_id();
        let comments =
            finish_comment_page(connection, comment_rows, viewer, &[owner_user_id]).await?;
        Ok(PhotographDetail {
            photograph: record.into(),
            comments,
            vote_state: vote_state(viewer_vote),
            owner_user_id,
        })
    }
}

fn vote_state(value: Option<bool>) -> VoteState {
    match value {
        Some(true) => VoteState::Upvoted,
        Some(false) => VoteState::Downvoted,
        None => VoteState::DidNotVote,
    }
}
