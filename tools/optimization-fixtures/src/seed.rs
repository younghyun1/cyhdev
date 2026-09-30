//! Stable synthetic records cloned identically before every candidate process.

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use rust_be_template::{
    features::accounts::domain::role::RoleType,
    schema::{forum_topics, media_object_cleanup, photographs, posts, user_roles, users},
};
use serde_json::{Value, json};
use uuid::Uuid;
use zeroize::Zeroizing;

pub const MEMBER: Uuid = Uuid::from_u128(0x01990000000070008000000000000001);
pub const ADMIN: Uuid = Uuid::from_u128(0x01990000000070008000000000000002);
pub const OTHER: Uuid = Uuid::from_u128(0x01990000000070008000000000000003);
pub const POST: Uuid = Uuid::from_u128(0x01990000000070008000000000000010);
pub const TOPIC: Uuid = Uuid::from_u128(0x01990000000070008000000000000020);
pub const PHOTO: Uuid = Uuid::from_u128(0x01990000000070008000000000000030);
pub const CLEANUP: Uuid = Uuid::from_u128(0x01990000000070008000000000000040);

pub async fn seed(connection: &mut AsyncPgConnection) -> anyhow::Result<Value> {
    let instant: DateTime<Utc> = "2026-09-29T00:00:00Z".parse()?;
    let geography = rust_be_template::schema::iso_country::table
        .filter(rust_be_template::schema::iso_country::is_country.eq(true))
        .order(rust_be_template::schema::iso_country::country_code.asc())
        .select((
            rust_be_template::schema::iso_country::country_code,
            rust_be_template::schema::iso_country::country_primary_language,
        ))
        .first::<(i32, i32)>(connection)
        .await?;
    for (id, label, role) in [
        (MEMBER, "fixture-member", RoleType::User),
        (ADMIN, "fixture-admin", RoleType::Younghyun),
        (OTHER, "fixture-other", RoleType::User),
    ] {
        let password = rust_be_template::util::crypto::hash_pw::hash_pw(Zeroizing::new(
            "OptimizationFixture123".to_owned(),
        ))
        .await?;
        diesel::insert_into(users::table)
            .values((
                users::user_id.eq(id),
                users::user_name.eq(match id {
                    MEMBER => "FixtureMember",
                    ADMIN => "FixtureAdmin",
                    _ => "FixtureOther",
                }),
                users::user_email.eq(format!("{label}@example.test")),
                users::user_password_hash.eq(password),
                users::user_is_email_verified.eq(true),
                users::user_country.eq(geography.0),
                users::user_language.eq(geography.1),
                users::user_created_at.eq(instant),
                users::user_updated_at.eq(instant),
            ))
            .execute(connection)
            .await?;
        diesel::insert_into(user_roles::table)
            .values((
                user_roles::user_id.eq(id),
                user_roles::role_id.eq(role.id()),
            ))
            .execute(connection)
            .await?;
    }
    diesel::insert_into(posts::table)
        .values((
            posts::post_id.eq(POST),
            posts::user_id.eq(ADMIN),
            posts::post_title.eq("Synthetic optimization fixture"),
            posts::post_slug.eq("synthetic-optimization-fixture"),
            posts::post_content
                .eq("# Synthetic optimization fixture\n\nDeterministic terrain and media checks."),
            posts::post_is_published.eq(true),
            posts::post_created_at.eq(instant),
            posts::post_updated_at.eq(instant),
            posts::post_published_at.eq(Some(instant)),
        ))
        .execute(connection)
        .await?;
    diesel::insert_into(forum_topics::table)
        .values((
            forum_topics::forum_topic_id.eq(TOPIC),
            forum_topics::forum_topic_author_user_id.eq(MEMBER),
            forum_topics::forum_topic_title.eq("Synthetic optimization discussion"),
            forum_topics::forum_topic_body.eq("Synthetic topic with deterministic fixture data."),
            forum_topics::forum_topic_created_at.eq(instant),
            forum_topics::forum_topic_updated_at.eq(instant),
            forum_topics::forum_topic_last_activity_at.eq(instant),
        ))
        .execute(connection)
        .await?;
    diesel::insert_into(photographs::table)
        .values((
            photographs::photograph_id.eq(PHOTO),
            photographs::user_id.eq(ADMIN),
            photographs::photograph_image_type.eq(1),
            photographs::photograph_is_on_cloud.eq(false),
            photographs::photograph_link
                .eq("https://127.0.0.1:18443/minecraft/map/tiles/fixture.png"),
            photographs::photograph_thumbnail_link
                .eq("https://127.0.0.1:18443/minecraft/map/tiles/fixture.png"),
            photographs::photograph_comments.eq("Synthetic optimization photograph"),
            photographs::photograph_lat.eq(0.0),
            photographs::photograph_lon.eq(0.0),
            photographs::photograph_created_at.eq(instant),
            photographs::photograph_updated_at.eq(instant),
        ))
        .execute(connection)
        .await?;
    diesel::insert_into(media_object_cleanup::table)
        .values((
            media_object_cleanup::media_object_cleanup_id.eq(CLEANUP),
            media_object_cleanup::media_object_cleanup_source_id.eq(PHOTO),
            media_object_cleanup::media_object_cleanup_original_url
                .eq("s3://cyhdev-img/images/fixture-cleanup.avif"),
            media_object_cleanup::media_object_cleanup_reason.eq("deleted_photograph_image"),
        ))
        .execute(connection)
        .await?;
    crate::retention::seed(connection, geography.0, geography.1).await?;
    Ok(
        json!({"purge_user_id":crate::retention::PURGE,"retained_user_id":crate::retention::DELETED,"notification_id":crate::retention::NOTICE,"post_id":POST,"topic_id":TOPIC,"photograph_id":PHOTO,"cleanup_id":CLEANUP,"member_id":MEMBER,"admin_id":ADMIN,"other_id":OTHER,
        "userName":"FixtureMember","user_name":"FixtureMember","country_id":geography.0.to_string(),"language_id":geography.1.to_string()}),
    )
}
