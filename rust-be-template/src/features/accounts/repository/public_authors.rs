//! Authoritative batched projections for retained public content.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::domain::public_author::PublicAuthor,
    schema::{user_profile_pictures, users},
};

type PublicAuthorRow = (Uuid, String, i32, Option<DateTime<Utc>>, Option<String>);

/// Load one deletion-aware author and latest-profile projection per user.
pub async fn load_public_authors(
    conn: &mut AsyncPgConnection,
    user_ids: &[Uuid],
) -> diesel::result::QueryResult<HashMap<Uuid, PublicAuthor>> {
    let mut requested_user_ids = user_ids.to_vec();
    requested_user_ids.sort_unstable();
    requested_user_ids.dedup();
    if requested_user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows: Vec<PublicAuthorRow> = users::table
        .left_join(user_profile_pictures::table.on(
            user_profile_pictures::user_id
                .eq(users::user_id)
                .and(user_profile_pictures::user_profile_picture_is_active.eq(true)),
        ))
        .filter(users::user_id.eq_any(&requested_user_ids))
        .select((
            users::user_id,
            users::user_name,
            users::user_country,
            users::user_deleted_at,
            user_profile_pictures::user_profile_picture_link.nullable(),
        ))
        .load(conn)
        .await?;

    let mut authors = HashMap::with_capacity(requested_user_ids.len());
    for (user_id, user_name, country_code, deleted_at, profile_picture_url) in rows {
        let author = if deleted_at.is_some() {
            PublicAuthor::deleted()
        } else {
            PublicAuthor::active(user_id, user_name, country_code, profile_picture_url)
        };
        authors.insert(user_id, author);
    }
    for user_id in requested_user_ids {
        authors.entry(user_id).or_insert_with(PublicAuthor::deleted);
    }
    Ok(authors)
}

/// Load only deletion state for response shapes without author badges.
pub async fn load_deleted_user_ids(
    conn: &mut AsyncPgConnection,
    user_ids: &[Uuid],
) -> diesel::result::QueryResult<HashSet<Uuid>> {
    let mut requested_user_ids = user_ids.to_vec();
    requested_user_ids.sort_unstable();
    requested_user_ids.dedup();
    if requested_user_ids.is_empty() {
        return Ok(HashSet::new());
    }
    let rows = users::table
        .filter(users::user_id.eq_any(&requested_user_ids))
        .select((users::user_id, users::user_deleted_at))
        .load::<(Uuid, Option<DateTime<Utc>>)>(conn)
        .await?;
    let mut deleted_user_ids = requested_user_ids.into_iter().collect::<HashSet<_>>();
    for (user_id, deleted_at) in rows {
        if deleted_at.is_none() {
            deleted_user_ids.remove(&user_id);
        }
    }
    Ok(deleted_user_ids)
}
