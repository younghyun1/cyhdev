//! PostgreSQL coverage for photograph vote-total serialization.

mod support;

use std::sync::Arc;

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use tokio::sync::Barrier;
use uuid::Uuid;

use rust_be_template::{
    features::photography::repository::{
        enums::DbPhotographContext, photography_repository::PhotographyRepository,
    },
    schema::{photograph_comment_votes, photograph_comments, photograph_votes, photographs},
};

use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::{account_test_context, seed_account},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn concurrent_votes_leave_exact_denormalized_totals() -> TestResult {
    run_database_test(concurrent_vote_case).await
}

fn concurrent_vote_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let owner = seed_account(&context, "PhotographVoteOwner").await?;
        let first_voter = seed_account(&context, "PhotographVoteOne").await?;
        let second_voter = seed_account(&context, "PhotographVoteTwo").await?;
        let mut connection = context.pool.get().await?;
        let photograph_id = diesel::insert_into(photographs::table)
            .values((
                photographs::user_id.eq(owner.user_id),
                photographs::photograph_shot_at.eq(None::<chrono::DateTime<chrono::Utc>>),
                photographs::photograph_image_type.eq(4),
                photographs::photograph_context.eq(DbPhotographContext::Photography),
                photographs::photograph_is_on_cloud.eq(true),
                photographs::photograph_link
                    .eq("https://objects.example.test/photography/vote.avif"),
                photographs::photograph_comments.eq("vote fixture"),
                photographs::photograph_lat.eq(0.0),
                photographs::photograph_lon.eq(0.0),
                photographs::photograph_thumbnail_link
                    .eq("https://objects.example.test/photography/vote-thumb.avif"),
            ))
            .returning(photographs::photograph_id)
            .get_result::<Uuid>(&mut connection)
            .await?;
        let comment_id = diesel::insert_into(photograph_comments::table)
            .values((
                photograph_comments::photograph_id.eq(photograph_id),
                photograph_comments::user_id.eq(owner.user_id),
                photograph_comments::photograph_comment_content.eq("vote fixture comment"),
            ))
            .returning(photograph_comments::photograph_comment_id)
            .get_result::<Uuid>(&mut connection)
            .await?;
        drop(connection);

        let repository = Arc::new(PhotographyRepository::new(context.pool.clone()));
        race_photograph_votes(
            Arc::clone(&repository),
            photograph_id,
            first_voter.user_id,
            second_voter.user_id,
        )
        .await?;
        race_comment_votes(
            Arc::clone(&repository),
            comment_id,
            first_voter.user_id,
            second_voter.user_id,
        )
        .await?;

        let mut connection = context.pool.get().await?;
        let stored_photograph_total = photographs::table
            .find(photograph_id)
            .select(photographs::photograph_total_upvotes)
            .first::<i64>(&mut connection)
            .await?;
        let photograph_vote_rows = photograph_votes::table
            .filter(photograph_votes::photograph_id.eq(photograph_id))
            .filter(photograph_votes::is_upvote.eq(true))
            .count()
            .get_result::<i64>(&mut connection)
            .await?;
        require(
            stored_photograph_total == photograph_vote_rows && photograph_vote_rows == 2,
            "concurrent photograph votes left a stale denormalized total",
        )?;

        let stored_comment_total = photograph_comments::table
            .find(comment_id)
            .select(photograph_comments::photograph_comment_total_upvotes)
            .first::<i64>(&mut connection)
            .await?;
        let comment_vote_rows = photograph_comment_votes::table
            .filter(photograph_comment_votes::photograph_comment_id.eq(comment_id))
            .filter(photograph_comment_votes::is_upvote.eq(true))
            .count()
            .get_result::<i64>(&mut connection)
            .await?;
        require(
            stored_comment_total == comment_vote_rows && comment_vote_rows == 2,
            "concurrent photograph-comment votes left a stale denormalized total",
        )
    })
}

async fn race_photograph_votes(
    repository: Arc<PhotographyRepository>,
    photograph_id: Uuid,
    first_user_id: Uuid,
    second_user_id: Uuid,
) -> TestResult {
    let barrier = Arc::new(Barrier::new(3));
    let first = spawn_photograph_vote(
        Arc::clone(&repository),
        Arc::clone(&barrier),
        photograph_id,
        first_user_id,
    );
    let second = spawn_photograph_vote(
        repository,
        Arc::clone(&barrier),
        photograph_id,
        second_user_id,
    );
    barrier.wait().await;
    first.await??;
    second.await??;
    Ok(())
}

fn spawn_photograph_vote(
    repository: Arc<PhotographyRepository>,
    barrier: Arc<Barrier>,
    photograph_id: Uuid,
    user_id: Uuid,
) -> tokio::task::JoinHandle<
    Result<(), rust_be_template::features::photography::error::PhotographyError>,
> {
    tokio::spawn(async move {
        barrier.wait().await;
        repository
            .vote_photograph(user_id, photograph_id, true)
            .await?;
        Ok(())
    })
}

async fn race_comment_votes(
    repository: Arc<PhotographyRepository>,
    comment_id: Uuid,
    first_user_id: Uuid,
    second_user_id: Uuid,
) -> TestResult {
    let barrier = Arc::new(Barrier::new(3));
    let first_repository = Arc::clone(&repository);
    let first_barrier = Arc::clone(&barrier);
    let first = tokio::spawn(async move {
        first_barrier.wait().await;
        first_repository
            .vote_comment(first_user_id, comment_id, true)
            .await
    });
    let second_barrier = Arc::clone(&barrier);
    let second = tokio::spawn(async move {
        second_barrier.wait().await;
        repository
            .vote_comment(second_user_id, comment_id, true)
            .await
    });
    barrier.wait().await;
    first.await??;
    second.await??;
    Ok(())
}
