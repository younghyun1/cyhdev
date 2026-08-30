//! Profile-picture metadata persistence.

use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::account::ProfilePictureReplacement,
        error::AccountError,
        repository::{account_repository::AccountRepository, records::NewProfilePictureRecord},
    },
    schema::{user_profile_pictures, users},
};

impl AccountRepository {
    /// Commits new metadata and retires prior links in one user-serialized transaction.
    pub async fn replace_profile_picture(
        &self,
        user_id: Uuid,
        image_type: i32,
        is_on_cloud: bool,
        link: Option<&str>,
    ) -> Result<ProfilePictureReplacement, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<ProfilePictureReplacement, diesel::result::Error, _>(
                async |connection| {
                    // Locking the account serializes even its first two picture uploads,
                    // when no profile-picture row exists to lock yet.
                    users::table
                        .find(user_id)
                        .select(users::user_id)
                        .for_update()
                        .first::<Uuid>(&mut *connection)
                        .await?;

                    let superseded_links = user_profile_pictures::table
                        .filter(user_profile_pictures::user_id.eq(user_id))
                        .filter(user_profile_pictures::user_profile_picture_is_on_cloud.eq(true))
                        .select(user_profile_pictures::user_profile_picture_link)
                        .load::<Option<String>>(&mut *connection)
                        .await?
                        .into_iter()
                        .flatten()
                        .collect();

                    let profile_picture_id = diesel::insert_into(user_profile_pictures::table)
                        .values(NewProfilePictureRecord {
                            user_id,
                            user_profile_picture_image_type: image_type,
                            user_profile_picture_is_on_cloud: is_on_cloud,
                            user_profile_picture_link: link,
                        })
                        .returning(user_profile_pictures::user_profile_picture_id)
                        .get_result(&mut *connection)
                        .await?;

                    diesel::update(
                        user_profile_pictures::table
                            .filter(user_profile_pictures::user_id.eq(user_id))
                            .filter(
                                user_profile_pictures::user_profile_picture_id
                                    .ne(profile_picture_id),
                            )
                            .filter(
                                user_profile_pictures::user_profile_picture_is_on_cloud.eq(true),
                            ),
                    )
                    .set((
                        user_profile_pictures::user_profile_picture_is_on_cloud.eq(false),
                        user_profile_pictures::user_profile_picture_link.eq(Option::<String>::None),
                        user_profile_pictures::user_profile_picture_updated_at.eq(Utc::now()),
                    ))
                    .execute(&mut *connection)
                    .await?;

                    Ok(ProfilePictureReplacement {
                        profile_picture_id,
                        superseded_links,
                    })
                },
            )
            .await
            .map_err(AccountError::Mutation)
    }
}
