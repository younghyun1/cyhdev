//! Geo-adjacent persistence kept outside lookup domain logic.

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::bb8::Pool};
use uuid::Uuid;

use crate::schema::user_profile_pictures;

pub struct GeoRepository {
    pool: Pool<AsyncPgConnection>,
}

impl GeoRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }

    pub async fn active_profile_picture(&self, user_id: Uuid) -> anyhow::Result<Option<String>> {
        let mut connection = self.pool.get().await?;
        Ok(user_profile_pictures::table
            .filter(user_profile_pictures::user_id.eq(user_id))
            .filter(user_profile_pictures::user_profile_picture_is_active.eq(true))
            .select(user_profile_pictures::user_profile_picture_link)
            .first::<Option<String>>(&mut connection)
            .await
            .optional()?
            .flatten())
    }
}
