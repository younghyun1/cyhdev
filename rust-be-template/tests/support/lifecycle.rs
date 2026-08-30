//! Authored-content fixtures for account lifecycle integration tests.

use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use rust_be_template::{
    features::{
        blog::repository::compatibility::{NewCommentVote, NewPost, NewPostVote},
        live_chat::{
            domain::message::LIVE_CHAT_SENDER_KIND_USER,
            repository::compatibility::LiveChatMessageInsertable,
        },
        photography::repository::enums::DbPhotographContext,
    },
    schema::{
        comment_votes, comments, live_chat_messages, photograph_comment_votes, photograph_comments,
        photograph_votes, photographs, post_votes, posts,
    },
};

use super::{
    database::{TestResult, require},
    fixtures::AccountTestContext,
};

pub const PROFILE_OBJECT_URL: &str = "https://objects.example.test/profile/lifecycle.avif";

pub struct AuthoredContentFixture {
    pub post_id: Uuid,
    pub comment_id: Uuid,
    pub photograph_id: Uuid,
    pub photograph_comment_id: Uuid,
    pub live_chat_message_id: Uuid,
}

pub async fn seed_authored_content(
    context: &AccountTestContext,
    user_id: Uuid,
    original_display_name: &str,
) -> TestResult<AuthoredContentFixture> {
    context
        .repository
        .replace_profile_picture(user_id, 4, true, Some(PROFILE_OBJECT_URL))
        .await?;

    let now = Utc::now();
    let metadata = serde_json::json!({"lifecycle_test": true});
    let post_slug = format!("retained-lifecycle-{user_id}");
    let mut connection = context.pool.get().await?;
    let post_id = diesel::insert_into(posts::table)
        .values(NewPost::new(
            &user_id,
            "Retained lifecycle post",
            &post_slug,
            "retained post body",
            Some(now),
            true,
            &metadata,
        ))
        .returning(posts::post_id)
        .get_result::<Uuid>(&mut connection)
        .await?;
    let comment_id = diesel::insert_into(comments::table)
        .values((
            comments::post_id.eq(post_id),
            comments::user_id.eq(user_id),
            comments::comment_content.eq("retained comment body"),
            comments::comment_created_at.eq(now),
        ))
        .returning(comments::comment_id)
        .get_result::<Uuid>(&mut connection)
        .await?;

    diesel::insert_into(post_votes::table)
        .values(NewPostVote::new(&post_id, &user_id, true))
        .execute(&mut connection)
        .await?;
    diesel::insert_into(comment_votes::table)
        .values(NewCommentVote::new(&comment_id, &user_id, true))
        .execute(&mut connection)
        .await?;
    diesel::update(posts::table.find(post_id))
        .set(posts::total_upvotes.eq(1_i64))
        .execute(&mut connection)
        .await?;
    diesel::update(comments::table.find(comment_id))
        .set(comments::total_upvotes.eq(1_i64))
        .execute(&mut connection)
        .await?;

    let photograph_id = diesel::insert_into(photographs::table)
        .values((
            photographs::user_id.eq(user_id),
            photographs::photograph_shot_at.eq(Some(now)),
            photographs::photograph_image_type.eq(4),
            photographs::photograph_context.eq(DbPhotographContext::Photography),
            photographs::photograph_is_on_cloud.eq(true),
            photographs::photograph_link.eq("https://objects.example.test/photo/lifecycle.avif"),
            photographs::photograph_comments.eq("retained photograph body"),
            photographs::photograph_lat.eq(37.0),
            photographs::photograph_lon.eq(-122.0),
            photographs::photograph_thumbnail_link
                .eq("https://objects.example.test/photo/lifecycle-thumb.avif"),
        ))
        .returning(photographs::photograph_id)
        .get_result::<Uuid>(&mut connection)
        .await?;
    let photograph_comment_id = diesel::insert_into(photograph_comments::table)
        .values((
            photograph_comments::photograph_id.eq(photograph_id),
            photograph_comments::user_id.eq(user_id),
            photograph_comments::photograph_comment_content.eq("retained photograph comment"),
            photograph_comments::parent_photograph_comment_id.eq(Option::<Uuid>::None),
        ))
        .returning(photograph_comments::photograph_comment_id)
        .get_result::<Uuid>(&mut connection)
        .await?;
    diesel::insert_into(photograph_votes::table)
        .values((
            photograph_votes::photograph_id.eq(photograph_id),
            photograph_votes::user_id.eq(user_id),
            photograph_votes::is_upvote.eq(true),
        ))
        .execute(&mut connection)
        .await?;
    diesel::insert_into(photograph_comment_votes::table)
        .values((
            photograph_comment_votes::photograph_comment_id.eq(photograph_comment_id),
            photograph_comment_votes::user_id.eq(user_id),
            photograph_comment_votes::is_upvote.eq(true),
        ))
        .execute(&mut connection)
        .await?;
    diesel::update(photographs::table.find(photograph_id))
        .set(photographs::photograph_total_upvotes.eq(1_i64))
        .execute(&mut connection)
        .await?;
    diesel::update(photograph_comments::table.find(photograph_comment_id))
        .set(photograph_comments::photograph_comment_total_upvotes.eq(1_i64))
        .execute(&mut connection)
        .await?;

    let live_chat_message_id = Uuid::now_v7();
    diesel::insert_into(live_chat_messages::table)
        .values(LiveChatMessageInsertable {
            live_chat_message_id,
            room_key: "main".to_owned(),
            user_id: Some(user_id),
            guest_ip: None,
            sender_kind: LIVE_CHAT_SENDER_KIND_USER,
            sender_display_name: original_display_name.to_owned(),
            message_body: "retained chat body".to_owned(),
            message_created_at: now,
        })
        .execute(&mut connection)
        .await?;
    drop(connection);

    Ok(AuthoredContentFixture {
        post_id,
        comment_id,
        photograph_id,
        photograph_comment_id,
        live_chat_message_id,
    })
}

pub async fn require_authored_content_retained(
    context: &AccountTestContext,
    fixture: &AuthoredContentFixture,
    user_id: Uuid,
    expected_chat_name: &str,
) -> TestResult {
    let mut connection = context.pool.get().await?;
    let post = posts::table
        .find(fixture.post_id)
        .select((posts::user_id, posts::post_content, posts::total_upvotes))
        .first::<(Uuid, String, i64)>(&mut connection)
        .await?;
    let comment = comments::table
        .find(fixture.comment_id)
        .select((
            comments::user_id,
            comments::comment_content,
            comments::total_upvotes,
        ))
        .first::<(Uuid, String, i64)>(&mut connection)
        .await?;
    let photograph = photographs::table
        .find(fixture.photograph_id)
        .select((
            photographs::user_id,
            photographs::photograph_comments,
            photographs::photograph_total_upvotes,
        ))
        .first::<(Uuid, String, i64)>(&mut connection)
        .await?;
    let photograph_comment = photograph_comments::table
        .find(fixture.photograph_comment_id)
        .select((
            photograph_comments::user_id,
            photograph_comments::photograph_comment_content,
            photograph_comments::photograph_comment_total_upvotes,
        ))
        .first::<(Uuid, String, i64)>(&mut connection)
        .await?;
    let chat = live_chat_messages::table
        .find(fixture.live_chat_message_id)
        .select((
            live_chat_messages::user_id,
            live_chat_messages::sender_display_name,
            live_chat_messages::message_body,
        ))
        .first::<(Option<Uuid>, String, String)>(&mut connection)
        .await?;
    let vote_counts = (
        post_votes::table
            .filter(post_votes::post_id.eq(fixture.post_id))
            .count()
            .get_result::<i64>(&mut connection)
            .await?,
        comment_votes::table
            .filter(comment_votes::comment_id.eq(fixture.comment_id))
            .count()
            .get_result::<i64>(&mut connection)
            .await?,
        photograph_votes::table
            .filter(photograph_votes::photograph_id.eq(fixture.photograph_id))
            .count()
            .get_result::<i64>(&mut connection)
            .await?,
        photograph_comment_votes::table
            .filter(
                photograph_comment_votes::photograph_comment_id.eq(fixture.photograph_comment_id),
            )
            .count()
            .get_result::<i64>(&mut connection)
            .await?,
    );
    drop(connection);

    require(
        post == (user_id, "retained post body".to_owned(), 1),
        "post was not retained",
    )?;
    require(
        comment == (user_id, "retained comment body".to_owned(), 1),
        "comment was not retained",
    )?;
    require(
        photograph == (user_id, "retained photograph body".to_owned(), 1),
        "photograph was not retained",
    )?;
    require(
        photograph_comment == (user_id, "retained photograph comment".to_owned(), 1),
        "photograph comment was not retained",
    )?;
    require(
        chat == (
            Some(user_id),
            expected_chat_name.to_owned(),
            "retained chat body".to_owned(),
        ),
        "chat history was not retained with the expected identity",
    )?;
    require(vote_counts == (1, 1, 1, 1), "vote rows were not retained")
}
