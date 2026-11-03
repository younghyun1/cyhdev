//! Comment tombstones, reply retention, and keyset comment pages.

mod support;

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use rust_be_template::{
    features::{
        blog::domain::comment_page::CommentPageRequest,
        photography::{
            domain::social::NewPhotographComment,
            repository::photography_repository::PhotographyRepository,
        },
    },
    schema::{comments, photograph_comments},
};

use support::{
    content::{blog_service, save_post, seed_photograph, verified_account},
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::account_test_context,
};

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn deleting_a_blog_comment_keeps_other_users_replies() -> TestResult {
    run_database_test(blog_tombstone_case).await
}

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn deleting_a_photograph_comment_keeps_other_users_replies() -> TestResult {
    run_database_test(photograph_tombstone_case).await
}

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn comment_pages_return_every_comment_once_in_order() -> TestResult {
    run_database_test(comment_page_case).await
}

fn blog_tombstone_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let author = verified_account(&context, "ThreadAuthor", true).await?;
        let replier = verified_account(&context, "ThreadReplier", false).await?;
        let blog = blog_service(context.pool.clone())?;
        let post_id = save_post(&blog, author.user_id, "Thread fixture", true).await?;
        let parent = blog
            .submit_comment(author.user_id, post_id, None, "Parent".to_owned())
            .await?;
        let reply = blog
            .submit_comment(
                replier.user_id,
                post_id,
                Some(parent.comment_id),
                "Reply".to_owned(),
            )
            .await?;
        blog.delete_comment(author.user_id, parent.comment_id)
            .await?;
        // A repeated delete is an idempotent success, not a second mutation.
        blog.delete_comment(author.user_id, parent.comment_id)
            .await?;
        let replies_to_tombstone = blog
            .submit_comment(
                replier.user_id,
                post_id,
                Some(parent.comment_id),
                "Late reply".to_owned(),
            )
            .await;
        require(
            replies_to_tombstone.is_err(),
            "a tombstone accepted a new reply",
        )?;

        let page = blog
            .comment_page(post_id, None, CommentPageRequest::first_page())
            .await?;
        let tombstone = page
            .comments
            .iter()
            .find(|comment| comment.comment_id == parent.comment_id);
        require(
            tombstone.is_some_and(|comment| {
                comment.comment_content.is_empty() && comment.comment_deleted_at.is_some()
            }),
            "deleted parent is not presented as an empty tombstone",
        )?;
        require(
            page.comments.iter().any(|comment| {
                comment.comment_id == reply.comment_id
                    && comment.parent_comment_id == Some(parent.comment_id)
                    && comment.comment_content == "Reply"
            }),
            "reply lost its content or parent after the parent was deleted",
        )?;

        let mut connection = context.pool.get().await?;
        let hard_delete = diesel::delete(comments::table.find(parent.comment_id))
            .execute(&mut connection)
            .await;
        require(
            hard_delete.is_err(),
            "the parent foreign key still allows deleting a replied-to comment",
        )?;
        drop(connection);

        // Deleting the post still removes its whole thread in one cascade.
        blog.delete_post(author.user_id, post_id).await?;
        let mut connection = context.pool.get().await?;
        let remaining = comments::table
            .filter(comments::post_id.eq(post_id))
            .count()
            .get_result::<i64>(&mut connection)
            .await?;
        require(remaining == 0, "post deletion left thread rows behind")
    })
}

fn photograph_tombstone_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let author = verified_account(&context, "PhotoThreadAuthor", false).await?;
        let replier = verified_account(&context, "PhotoThreadReplier", false).await?;
        let photograph_id = seed_photograph(&context.pool, author.user_id).await?;
        let repository = PhotographyRepository::new(context.pool.clone());
        let parent = repository
            .create_comment(NewPhotographComment {
                photograph_id,
                user_id: author.user_id,
                content: "Parent".to_owned(),
                parent_comment_id: None,
            })
            .await?
            .comment;
        let reply = repository
            .create_comment(NewPhotographComment {
                photograph_id,
                user_id: replier.user_id,
                content: "Reply".to_owned(),
                parent_comment_id: Some(parent.photograph_comment_id),
            })
            .await?
            .comment;
        repository
            .delete_comment(author.user_id, parent.photograph_comment_id)
            .await?;
        let vote = repository
            .vote_comment(replier.user_id, parent.photograph_comment_id, true)
            .await;
        require(vote.is_err(), "a tombstone accepted a new vote")?;

        let mut connection = context.pool.get().await?;
        let rows = photograph_comments::table
            .filter(photograph_comments::photograph_id.eq(photograph_id))
            .select((
                photograph_comments::photograph_comment_id,
                photograph_comments::photograph_comment_content,
                photograph_comments::photograph_comment_deleted_at,
                photograph_comments::parent_photograph_comment_id,
            ))
            .load::<(Uuid, String, Option<DateTime<Utc>>, Option<Uuid>)>(&mut connection)
            .await?;
        require(
            rows.iter().any(|row| {
                row.0 == parent.photograph_comment_id && row.1.is_empty() && row.2.is_some()
            }),
            "deleted photograph comment is not a tombstone",
        )?;
        require(
            rows.iter().any(|row| {
                row.0 == reply.photograph_comment_id
                    && row.1 == "Reply"
                    && row.3 == Some(parent.photograph_comment_id)
            }),
            "photograph reply did not survive its parent's deletion",
        )?;
        let hard_delete =
            diesel::delete(photograph_comments::table.find(parent.photograph_comment_id))
                .execute(&mut connection)
                .await;
        require(
            hard_delete.is_err(),
            "the photograph parent foreign key still cascades",
        )
    })
}

fn comment_page_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let author = verified_account(&context, "PageAuthor", true).await?;
        let blog = blog_service(context.pool.clone())?;
        let post_id = save_post(&blog, author.user_id, "Paged thread", true).await?;
        let tied_at = Utc::now();
        let mut connection = context.pool.get().await?;
        // Seven rows share one timestamp so only the id tie-breaker orders them.
        for index in 0..7 {
            diesel::insert_into(comments::table)
                .values((
                    comments::post_id.eq(post_id),
                    comments::user_id.eq(author.user_id),
                    comments::comment_content.eq(format!("Tied comment {index}")),
                    comments::comment_created_at.eq(tied_at),
                ))
                .execute(&mut connection)
                .await?;
        }
        drop(connection);

        let mut seen = Vec::new();
        let mut request = CommentPageRequest::parse(None, None, Some(3))?;
        let mut pages = 0;
        loop {
            let page = blog.comment_page(post_id, None, request).await?;
            pages += 1;
            seen.extend(page.comments.iter().map(|comment| comment.comment_id));
            match page.next_cursor {
                Some(cursor) => {
                    request = CommentPageRequest::parse(
                        Some(cursor.created_at),
                        Some(cursor.comment_id),
                        Some(3),
                    )?;
                }
                None => break,
            }
            require(pages <= 3, "comment pagination did not terminate")?;
        }
        let unique = seen.iter().copied().collect::<HashSet<_>>();
        let mut sorted = seen.clone();
        sorted.sort_unstable();
        require(
            seen.len() == 7 && unique.len() == 7 && sorted == seen && pages == 3,
            "tied comment pages skipped, repeated, or reordered rows",
        )
    })
}
