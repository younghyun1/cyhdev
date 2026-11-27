//! Visibility, absence, and viewer state on blog and forum reads that send
//! several statements at once. Each such read fetches data before it knows
//! whether the request is admissible, so every denied and missing outcome is
//! asserted alongside the enriched result.

use std::sync::Arc;

use uuid::Uuid;

use rust_be_template::features::{
    blog::{
        domain::{comment_page::CommentPageRequest, post::PostLookup, vote::VoteState},
        error::BlogError,
        repository::blog_repository::BlogRepository,
    },
    forum::{
        error::ForumError, repository::forum_repository::ForumRepository,
        service::forum_service::ForumService,
    },
};

use crate::support::{
    content::{blog_service, save_post, verified_account},
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::account_test_context,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn blog_reads_hide_drafts_and_carry_viewer_state() -> TestResult {
    run_database_test(blog_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn forum_topic_reads_report_absence_and_subscription() -> TestResult {
    run_database_test(forum_case).await
}

fn blog_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let manager = verified_account(&context, "ReadManager", true).await?;
        let visitor = verified_account(&context, "ReadVisitor", false).await?;
        let writer = blog_service(context.pool.clone())?;
        let published_id = save_post(&writer, manager.user_id, "Published read", true).await?;
        let draft_id = save_post(&writer, manager.user_id, "Draft read", false).await?;
        let comment = writer
            .submit_comment(manager.user_id, published_id, None, "First".to_owned())
            .await?;
        writer
            .vote_post(visitor.user_id, published_id, true)
            .await?;
        writer
            .vote_comment(visitor.user_id, comment.comment_id, false)
            .await?;
        // A second service has an empty metadata cache, so its detail reads
        // load tags from PostgreSQL next to the post itself.
        let blog = blog_service(context.pool.clone())?;

        for viewer in [None, Some(visitor.user_id)] {
            let read = blog.read_post(PostLookup::Id(draft_id), viewer).await;
            require(
                matches!(read, Err(BlogError::PostNotFound)),
                "a draft was readable without blog authority",
            )?;
            let page = blog
                .comment_page(draft_id, viewer, CommentPageRequest::first_page())
                .await;
            require(
                matches!(page, Err(BlogError::PostNotFound)),
                "a draft's comments were listed without blog authority",
            )?;
        }
        let managed = blog
            .read_post(PostLookup::Id(draft_id), Some(manager.user_id))
            .await?;
        require(
            managed.post.post_id == draft_id
                && !managed.post.post_is_published
                && managed.post_tags == ["fixture"],
            "the manager did not read the draft with its tags",
        )?;
        let managed_page = blog
            .comment_page(
                draft_id,
                Some(manager.user_id),
                CommentPageRequest::first_page(),
            )
            .await?;
        require(
            managed_page.comments.is_empty(),
            "a draft without comments returned a comment",
        )?;
        let missing_id = Uuid::now_v7();
        let missing = blog
            .read_post(PostLookup::Id(missing_id), Some(manager.user_id))
            .await;
        let missing_page = blog
            .comment_page(missing_id, None, CommentPageRequest::first_page())
            .await;
        require(
            matches!(missing, Err(BlogError::PostNotFound))
                && matches!(missing_page, Err(BlogError::PostNotFound)),
            "a missing post was not reported as absent",
        )?;

        let anonymous = blog.read_post(PostLookup::Id(published_id), None).await?;
        require(
            anonymous.post_tags == ["fixture"]
                && anonymous.comments.len() == 1
                && anonymous.vote_state == VoteState::DidNotVote
                && anonymous.comments[0].vote_state == VoteState::DidNotVote,
            "an anonymous detail read lost tags or comments, or carried a vote",
        )?;
        let viewed = blog
            .read_post(PostLookup::Id(published_id), Some(visitor.user_id))
            .await?;
        require(
            viewed.vote_state == VoteState::Upvoted
                && viewed.comments.len() == 1
                && viewed.comments[0].comment_id == comment.comment_id
                && viewed.comments[0].vote_state == VoteState::Downvoted,
            "a detail read did not carry the viewer's post and comment votes",
        )?;
        let page = blog
            .comment_page(
                published_id,
                Some(visitor.user_id),
                CommentPageRequest::first_page(),
            )
            .await?;
        require(
            page.comments.len() == 1 && page.comments[0].vote_state == VoteState::Downvoted,
            "a comment page did not carry the viewer's vote",
        )?;

        for viewer in [None, Some(visitor.user_id)] {
            let (posts, pages) = blog.list_posts(1, 1, viewer).await;
            require(
                posts.len() == 1 && posts[0].post_id == published_id && pages == 1,
                "a listing without blog authority counted or returned a draft",
            )?;
        }
        let (managed_posts, managed_pages) = blog.list_posts(1, 1, Some(manager.user_id)).await;
        let (all_posts, _) = blog.list_posts(1, 10, Some(manager.user_id)).await;
        require(
            managed_posts.len() == 1
                && managed_pages == 2
                && all_posts.len() == 2
                && all_posts.iter().any(|post| post.post_id == draft_id)
                && all_posts.iter().all(|post| post.post_tags == ["fixture"]),
            "the manager's listing did not count and return the draft with tags",
        )?;
        let presented = blog.present_posts(all_posts, Some(visitor.user_id)).await?;
        require(
            presented.len() == 2
                && presented
                    .iter()
                    .all(|post| post.user_name == manager.user_name)
                && presented.iter().any(|post| {
                    post.post_id == published_id && post.vote_state == VoteState::Upvoted
                })
                && presented.iter().any(|post| {
                    post.post_id == draft_id && post.vote_state == VoteState::DidNotVote
                }),
            "listing presentation lost the author or the viewer's vote",
        )?;
        let by_ids = BlogRepository::new(context.pool.clone())
            .posts_by_ids(&[draft_id, published_id])
            .await?;
        require(
            by_ids.len() == 2 && by_ids.iter().all(|post| post.post_tags == ["fixture"]),
            "a batch read by identifier lost a post or its tags",
        )
    })
}

fn forum_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let author = verified_account(&context, "TopicAuthor", false).await?;
        let replier = verified_account(&context, "TopicReplier", false).await?;
        let reader = verified_account(&context, "TopicReader", false).await?;
        let repository = Arc::new(ForumRepository::new(context.pool.clone()));
        let forum = ForumService::new(repository, Arc::clone(&context.accounts));
        let topic = forum
            .create_topic(
                author.user_id,
                "Joined topic read".to_owned(),
                "Opening body".to_owned(),
            )
            .await?;
        forum
            .create_reply(replier.user_id, topic.item_id, "A reply".to_owned())
            .await?;

        for viewer in [None, Some(reader.user_id)] {
            let missing = forum
                .topic(Uuid::now_v7(), viewer, None, None, Some(10))
                .await;
            require(
                matches!(missing, Err(ForumError::TopicNotFound)),
                "a missing topic was not reported as absent",
            )?;
        }
        let unsubscribed = forum
            .topic(topic.item_id, Some(reader.user_id), None, None, Some(10))
            .await?;
        require(
            !unsubscribed.is_subscribed
                && unsubscribed.topic.author.user_name() == author.user_name
                && unsubscribed.replies.items.len() == 1
                && unsubscribed.replies.items[0].author.user_name() == replier.user_name,
            "a topic read lost its authors or reported a subscription that does not exist",
        )?;
        forum
            .set_subscription(reader.user_id, topic.item_id, true)
            .await?;
        let subscribed = forum
            .topic(topic.item_id, Some(reader.user_id), None, None, Some(10))
            .await?;
        let anonymous = forum
            .topic(topic.item_id, None, None, None, Some(10))
            .await?;
        require(
            subscribed.is_subscribed && !anonymous.is_subscribed,
            "a topic read did not report the viewer's own subscription",
        )
    })
}
