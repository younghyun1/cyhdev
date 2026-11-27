//! Photograph and account reads that join statements or tables: absence,
//! the viewer's votes, deleted-owner masking, the single active profile
//! picture, and forum authority read in one statement.

use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use rust_be_template::{
    features::{
        accounts::authorization_error::AuthorizationError,
        blog::domain::{comment_page::CommentPageRequest, vote::VoteState},
        photography::{
            error::PhotographyError, repository::photography_repository::PhotographyRepository,
        },
    },
    schema::{photograph_comments, user_profile_pictures},
};

use crate::support::{
    content::{seed_photograph, verified_account},
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::{VALID_PASSWORD, account_test_context, seed_account},
};

const ACTIVE_PICTURE: &str = "https://objects.example.test/profile/active.avif";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn photograph_reads_report_absence_and_mask_a_deleted_owner() -> TestResult {
    run_database_test(photograph_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn account_reads_join_the_active_picture_and_current_authority() -> TestResult {
    run_database_test(account_case).await
}

fn photograph_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let owner = verified_account(&context, "DetailOwner", false).await?;
        let viewer = verified_account(&context, "DetailViewer", false).await?;
        let photograph_id = seed_photograph(&context.pool, owner.user_id).await?;
        let mut connection = context.pool.get().await?;
        let comment_id = diesel::insert_into(photograph_comments::table)
            .values((
                photograph_comments::photograph_id.eq(photograph_id),
                photograph_comments::user_id.eq(owner.user_id),
                photograph_comments::photograph_comment_content.eq("detail fixture comment"),
            ))
            .returning(photograph_comments::photograph_comment_id)
            .get_result::<Uuid>(&mut connection)
            .await?;
        drop(connection);
        let repository = PhotographyRepository::new(context.pool.clone());
        repository
            .vote_photograph(viewer.user_id, photograph_id, true)
            .await?;
        repository
            .vote_comment(viewer.user_id, comment_id, false)
            .await?;

        let missing_id = Uuid::now_v7();
        let missing = repository
            .photograph_detail(missing_id, Some(viewer.user_id))
            .await;
        let missing_page = repository
            .comment_page(
                missing_id,
                Some(viewer.user_id),
                CommentPageRequest::first_page(),
            )
            .await;
        require(
            matches!(missing, Err(PhotographyError::PhotographNotFound))
                && matches!(missing_page, Err(PhotographyError::PhotographNotFound)),
            "a missing photograph was not reported as absent",
        )?;

        let detail = repository
            .photograph_detail(photograph_id, Some(viewer.user_id))
            .await?;
        require(
            detail.owner_user_id == owner.user_id
                && detail.vote_state == VoteState::Upvoted
                && detail.comments.comments.len() == 1
                && detail.comments.comments[0].1 == VoteState::Downvoted
                && detail
                    .comments
                    .authors
                    .get(&owner.user_id)
                    .is_some_and(|author| !author.is_deleted()),
            "a detail read lost the owner projection or the viewer's votes",
        )?;
        let anonymous = repository.photograph_detail(photograph_id, None).await?;
        require(
            anonymous.vote_state == VoteState::DidNotVote
                && anonymous.comments.comments.len() == 1
                && anonymous.comments.comments[0].1 == VoteState::DidNotVote,
            "an anonymous detail read carried a vote",
        )?;
        let page = repository
            .comment_page(
                photograph_id,
                Some(viewer.user_id),
                CommentPageRequest::first_page(),
            )
            .await?;
        require(
            page.comments.len() == 1 && page.comments[0].1 == VoteState::Downvoted,
            "a comment page did not carry the viewer's vote",
        )?;
        let first = repository.photograph_page(1, 10).await?;
        let beyond = repository.photograph_page(2, 10).await?;
        require(
            first.total_items == 1
                && first.items.len() == 1
                && beyond.total_items == 1
                && beyond.items.is_empty(),
            "a photograph page disagreed with its total",
        )?;

        context
            .accounts
            .soft_delete_account(owner.user_id, VALID_PASSWORD)
            .await?;
        let masked = repository.photograph_detail(photograph_id, None).await?;
        require(
            masked
                .comments
                .authors
                .get(&owner.user_id)
                .is_some_and(|author| author.is_deleted()),
            "a deleted owner was not masked in the detail author batch",
        )
    })
}

fn account_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let account = verified_account(&context, "PictureOwner", false).await?;
        let manager = verified_account(&context, "PictureManager", true).await?;
        let unverified = seed_account(&context, "PictureUnverified").await?;

        let bare = context.repository.current_account(account.user_id).await?;
        let bare_public = context.accounts.public_account(&account.user_name).await?;
        require(
            bare.is_some_and(|current| {
                current.profile.user_id == account.user_id && current.profile_picture.is_none()
            }) && bare_public.profile_picture_url.is_none(),
            "an account without a picture was dropped or given one",
        )?;

        let mut connection = context.pool.get().await?;
        diesel::insert_into(user_profile_pictures::table)
            .values([
                (
                    user_profile_pictures::user_id.eq(account.user_id),
                    user_profile_pictures::user_profile_picture_image_type.eq(4),
                    user_profile_pictures::user_profile_picture_is_on_cloud.eq(true),
                    user_profile_pictures::user_profile_picture_is_active.eq(false),
                    user_profile_pictures::user_profile_picture_link
                        .eq("https://objects.example.test/profile/retired.avif"),
                ),
                (
                    user_profile_pictures::user_id.eq(account.user_id),
                    user_profile_pictures::user_profile_picture_image_type.eq(4),
                    user_profile_pictures::user_profile_picture_is_on_cloud.eq(true),
                    user_profile_pictures::user_profile_picture_is_active.eq(true),
                    user_profile_pictures::user_profile_picture_link.eq(ACTIVE_PICTURE),
                ),
            ])
            .execute(&mut connection)
            .await?;
        drop(connection);
        let current = context.repository.current_account(account.user_id).await?;
        let public = context.accounts.public_account(&account.user_name).await?;
        require(
            current.is_some_and(|current| {
                current.profile_picture.is_some_and(|picture| {
                    picture.is_active && picture.link.as_deref() == Some(ACTIVE_PICTURE)
                })
            }) && public.profile_picture_url.as_deref() == Some(ACTIVE_PICTURE),
            "the active picture was not joined, or a retired picture was",
        )?;
        require(
            context
                .repository
                .current_account(Uuid::now_v7())
                .await?
                .is_none(),
            "a missing account produced a profile",
        )?;

        for inactive in [Uuid::now_v7(), unverified.user_id] {
            let lease = context.accounts.acquire_forum_authority(inactive).await;
            require(
                matches!(lease, Err(AuthorizationError::AccountNotFound)),
                "a missing or unverified account received forum authority",
            )?;
        }
        let ordinary = context
            .accounts
            .acquire_forum_authority(account.user_id)
            .await?
            .authority();
        let moderating = context
            .accounts
            .acquire_forum_authority(manager.user_id)
            .await?
            .authority();
        require(
            ordinary.user_id == account.user_id
                && !ordinary.can_moderate
                && moderating.can_moderate,
            "forum authority did not match the account's current role",
        )
    })
}
