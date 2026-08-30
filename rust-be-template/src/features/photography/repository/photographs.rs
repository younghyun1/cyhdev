use diesel::{
    ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper,
    sql_types::{Array, BigInt, Uuid as SqlUuid},
};
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use tracing::warn;
use uuid::Uuid;

use crate::{
    features::blog::domain::vote::VoteState,
    features::photography::{
        domain::{
            photograph::{Photograph, PhotographDetail, PhotographPage},
            social::PhotographComment,
        },
        error::PhotographyError,
        repository::{
            enums::DbPhotographContext,
            photography_repository::PhotographyRepository,
            records::{PhotographCommentRecord, PhotographRecord},
        },
    },
    persistence::public_authors::{load_deleted_user_ids, load_public_authors},
    schema::{photograph_comment_votes, photograph_comments, photograph_votes, photographs},
};

pub const PHOTOGRAPH_DETAIL_COMMENT_LIMIT: i64 = 1_000;
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
        let mut connection = self.connection().await?;
        let total_items = photographs::table
            .filter(photographs::photograph_context.eq(DbPhotographContext::Photography))
            .count()
            .get_result::<i64>(&mut connection)
            .await?;
        let records = photographs::table
            .filter(photographs::photograph_context.eq(DbPhotographContext::Photography))
            .order((
                photographs::photograph_shot_at.desc(),
                photographs::photograph_id.desc(),
            ))
            .offset((page - 1).saturating_mul(page_size))
            .limit(page_size)
            .select(PhotographRecord::as_select())
            .load::<PhotographRecord>(&mut connection)
            .await?;
        let owner_ids = records
            .iter()
            .map(|record| record.clone_author_id())
            .collect::<Vec<_>>();
        let deleted = load_deleted_user_ids(&mut connection, &owner_ids).await?;
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

    pub async fn photograph_detail(
        &self,
        photograph_id: Uuid,
        viewer: Option<Uuid>,
    ) -> Result<PhotographDetail, PhotographyError> {
        let mut connection = self.connection().await?;
        let record = photographs::table
            .filter(photographs::photograph_id.eq(photograph_id))
            .select(PhotographRecord::as_select())
            .first::<PhotographRecord>(&mut connection)
            .await
            .optional()?
            .ok_or(PhotographyError::PhotographNotFound)?;
        let mut comment_records = photograph_comments::table
            .filter(photograph_comments::photograph_id.eq(photograph_id))
            .order((
                (photograph_comments::photograph_comment_total_upvotes
                    - photograph_comments::photograph_comment_total_downvotes)
                    .desc(),
                photograph_comments::photograph_comment_created_at.asc(),
                photograph_comments::photograph_comment_id.asc(),
            ))
            .limit(PHOTOGRAPH_DETAIL_COMMENT_LIMIT + 1)
            .select(PhotographCommentRecord::as_select())
            .load::<PhotographCommentRecord>(&mut connection)
            .await?;
        if comment_records.len() > PHOTOGRAPH_DETAIL_COMMENT_LIMIT as usize {
            warn!(%photograph_id, limit = PHOTOGRAPH_DETAIL_COMMENT_LIMIT, "Photograph detail comments were truncated at the fixed response ceiling");
            comment_records.truncate(PHOTOGRAPH_DETAIL_COMMENT_LIMIT as usize);
        }
        let comments = comment_records
            .into_iter()
            .map(PhotographComment::from)
            .collect::<Vec<_>>();
        let mut ids = comments
            .iter()
            .map(|comment| comment.user_id)
            .collect::<Vec<_>>();
        ids.push(record.clone_author_id());
        ids.sort_unstable();
        ids.dedup();
        let authors = load_public_authors(&mut connection, &ids).await?;
        let (comment_votes, vote_state) = match viewer {
            Some(user_id) => {
                let comment_ids = comments
                    .iter()
                    .map(|comment| comment.photograph_comment_id)
                    .collect::<Vec<_>>();
                let rows = photograph_comment_votes::table
                    .filter(photograph_comment_votes::photograph_comment_id.eq_any(&comment_ids))
                    .filter(photograph_comment_votes::user_id.eq(user_id))
                    .select((
                        photograph_comment_votes::photograph_comment_id,
                        photograph_comment_votes::is_upvote,
                    ))
                    .load::<(Uuid, bool)>(&mut connection)
                    .await?;
                let photograph_vote = photograph_votes::table
                    .filter(photograph_votes::photograph_id.eq(photograph_id))
                    .filter(photograph_votes::user_id.eq(user_id))
                    .select(photograph_votes::is_upvote)
                    .first::<bool>(&mut connection)
                    .await
                    .optional()?;
                (
                    rows.into_iter()
                        .map(|(id, vote)| (id, vote_state(Some(vote))))
                        .collect::<HashMap<_, _>>(),
                    vote_state(photograph_vote),
                )
            }
            None => (HashMap::new(), VoteState::DidNotVote),
        };
        let owner_user_id = record.clone_author_id();
        let photograph = record.into();
        let comments = comments
            .into_iter()
            .map(|comment| {
                let state = comment_votes
                    .get(&comment.photograph_comment_id)
                    .cloned()
                    .unwrap_or(VoteState::DidNotVote);
                (comment, state)
            })
            .collect();
        Ok(PhotographDetail {
            photograph,
            comments,
            vote_state,
            authors,
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
