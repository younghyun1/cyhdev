//! Blog and photograph fixtures built through public service boundaries.

use std::{collections::HashMap, sync::Arc};

use diesel::ExpressionMethods;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::bb8::Pool};
use uuid::Uuid;

use rust_be_template::{
    features::{
        accounts::domain::role::RoleType,
        blog::{
            domain::post::SavePostInput,
            repository::blog_repository::BlogRepository,
            service::{blog_service::BlogService, search::search_index::PostSearchIndex},
        },
        photography::repository::enums::DbPhotographContext,
        reference_data::service::reference_data_service::CountryFlagLookupPort,
    },
    schema::photographs,
};

use super::{
    database::TestResult,
    fixtures::{AccountFixture, AccountTestContext, seed_account},
};

pub struct EmptyCountryFlags;

#[async_trait::async_trait]
impl CountryFlagLookupPort for EmptyCountryFlags {
    async fn country_flag(&self, _country_code: i32) -> Option<String> {
        None
    }
    async fn country_flags(&self, _country_codes: &[i32]) -> HashMap<i32, String> {
        HashMap::new()
    }
}

pub fn blog_service(pool: Pool<AsyncPgConnection>) -> TestResult<BlogService> {
    let repository = Arc::new(BlogRepository::new(pool));
    let search = Arc::new(PostSearchIndex::new_in_memory()?);
    let flags: Arc<dyn CountryFlagLookupPort> = Arc::new(EmptyCountryFlags);
    Ok(BlogService::new(repository, search, flags))
}

/// A verified account; `manager` also receives the blog-managing role.
pub async fn verified_account(
    context: &AccountTestContext,
    label: &str,
    manager: bool,
) -> TestResult<AccountFixture> {
    let account = seed_account(context, label).await?;
    context
        .accounts
        .verify_email(&account.verification_token)
        .await?;
    if manager {
        context
            .accounts
            .assign_role(account.user_id, RoleType::Younghyun)
            .await?;
    }
    Ok(account)
}

pub async fn save_post(
    blog: &BlogService,
    author_id: Uuid,
    title: &str,
    published: bool,
) -> TestResult<Uuid> {
    let post = blog
        .save_post(SavePostInput {
            actor_user_id: author_id,
            post_id: None,
            title: title.to_owned(),
            markdown: "Fixture body.".to_owned(),
            tags: vec!["fixture".to_owned()],
            published,
            owner_required: true,
        })
        .await?;
    Ok(post.post_id)
}

pub async fn seed_photograph(pool: &Pool<AsyncPgConnection>, owner_id: Uuid) -> TestResult<Uuid> {
    let mut connection = pool.get().await?;
    let photograph_id = diesel::insert_into(photographs::table)
        .values((
            photographs::user_id.eq(owner_id),
            photographs::photograph_shot_at.eq(None::<chrono::DateTime<chrono::Utc>>),
            photographs::photograph_image_type.eq(4),
            photographs::photograph_context.eq(DbPhotographContext::Photography),
            photographs::photograph_is_on_cloud.eq(true),
            photographs::photograph_link.eq("https://objects.example.test/images/thread.avif"),
            photographs::photograph_comments.eq("thread fixture"),
            photographs::photograph_lat.eq(0.0),
            photographs::photograph_lon.eq(0.0),
            photographs::photograph_thumbnail_link
                .eq("https://objects.example.test/thumbnails/thread.avif"),
        ))
        .returning(photographs::photograph_id)
        .get_result::<Uuid>(&mut connection)
        .await?;
    Ok(photograph_id)
}
