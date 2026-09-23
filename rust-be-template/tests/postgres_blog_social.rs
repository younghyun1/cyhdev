//! Draft visibility for blog social writes and buffered view persistence.

mod support;

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

use rust_be_template::{
    features::blog::{
        domain::post::{PostLookup, SavePostInput},
        error::BlogError,
    },
    schema::{posts, tags},
};

use support::{
    content::{blog_service, save_post, verified_account},
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::account_test_context,
};

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn drafts_reject_comments_and_votes_from_non_managers() -> TestResult {
    run_database_test(draft_case).await
}

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn detail_views_are_buffered_until_flushed() -> TestResult {
    run_database_test(view_buffer_case).await
}

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn resaving_existing_tags_consumes_no_identity_values() -> TestResult {
    run_database_test(tag_identity_case).await
}

fn draft_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let manager = verified_account(&context, "DraftManager", true).await?;
        let visitor = verified_account(&context, "DraftVisitor", false).await?;
        let blog = blog_service(context.pool.clone())?;
        let draft_id = save_post(&blog, manager.user_id, "Unpublished draft", false).await?;

        let comment = blog
            .submit_comment(visitor.user_id, draft_id, None, "Too early".to_owned())
            .await;
        require(
            matches!(comment, Err(BlogError::PostNotFound)),
            "a non-manager commented on a draft",
        )?;
        let vote = blog.vote_post(visitor.user_id, draft_id, true).await;
        require(
            matches!(vote, Err(BlogError::PostNotFound)),
            "a non-manager voted on a draft",
        )?;
        let manager_comment = blog
            .submit_comment(manager.user_id, draft_id, None, "Review note".to_owned())
            .await?;
        let comment_vote = blog
            .vote_comment(visitor.user_id, manager_comment.comment_id, true)
            .await;
        require(
            matches!(comment_vote, Err(BlogError::PostNotFound)),
            "a non-manager voted on a draft's comment",
        )?;
        let page = blog
            .comment_page(
                draft_id,
                Some(visitor.user_id),
                rust_be_template::features::blog::domain::comment_page::CommentPageRequest::first_page(),
            )
            .await;
        require(
            matches!(page, Err(BlogError::PostNotFound)),
            "a non-manager listed a draft's comments",
        )?;
        let managed_vote = blog.vote_post(manager.user_id, draft_id, true).await?;
        require(
            managed_vote.upvotes == 1 && managed_vote.downvotes == 0,
            "the manager's draft vote was not tallied by the filtered aggregate",
        )
    })
}

fn view_buffer_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let author = verified_account(&context, "ViewAuthor", true).await?;
        let blog = blog_service(context.pool.clone())?;
        let post_id = save_post(&blog, author.user_id, "Viewed post", true).await?;

        let first = blog.read_post(PostLookup::Id(post_id), None).await?;
        let second = blog.read_post(PostLookup::Id(post_id), None).await?;
        require(
            first.post.post_view_count == 1 && second.post.post_view_count == 2,
            "presented view counts do not include buffered views",
        )?;
        let stored = stored_views(&context.pool, post_id).await?;
        require(stored == 0, "a detail read wrote the view counter directly")?;

        let flushed = blog.flush_views().await?;
        let stored = stored_views(&context.pool, post_id).await?;
        require(
            flushed == 2 && stored == 2,
            "flushing did not persist every buffered view",
        )?;
        let third = blog.read_post(PostLookup::Id(post_id), None).await?;
        require(
            third.post.post_view_count == 3 && blog.flush_views().await? == 1,
            "a flushed view was counted twice or lost",
        )
    })
}

fn tag_identity_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let author = verified_account(&context, "TagAuthor", true).await?;
        let blog = blog_service(context.pool.clone())?;
        let post_id = save_post(&blog, author.user_id, "Tagged post", true).await?;
        let resave = |tags: Vec<String>| SavePostInput {
            actor_user_id: author.user_id,
            post_id: Some(post_id),
            title: "Tagged post".to_owned(),
            markdown: "Fixture body.".to_owned(),
            tags,
            published: true,
            owner_required: true,
        };
        for _ in 0..3 {
            blog.save_post(resave(vec!["fixture".to_owned()])).await?;
        }
        blog.save_post(resave(vec!["fixture".to_owned(), "fresh".to_owned()]))
            .await?;
        let mut connection = context.pool.get().await?;
        let tag_ids = tags::table
            .order(tags::tag_id.asc())
            .select(tags::tag_id)
            .load::<i32>(&mut connection)
            .await?;
        require(
            tag_ids.len() == 2 && tag_ids[1] == tag_ids[0] + 1,
            "re-saving existing tags consumed identity values",
        )
    })
}

async fn stored_views(
    pool: &diesel_async::pooled_connection::bb8::Pool<diesel_async::AsyncPgConnection>,
    post_id: uuid::Uuid,
) -> TestResult<i64> {
    let mut connection = pool.get().await?;
    Ok(posts::table
        .filter(posts::post_id.eq(post_id))
        .select(posts::post_view_count)
        .first::<i64>(&mut connection)
        .await?)
}
