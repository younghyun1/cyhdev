//! PostgreSQL-backed evidence for process-cache publication and refresh.

mod support;

use std::{collections::HashMap, sync::Arc};

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

use rust_be_template::{
    features::{
        accounts::domain::role::RoleType,
        blog::{
            domain::post::SavePostInput,
            repository::blog_repository::BlogRepository,
            service::{blog_service::BlogService, search::search_index::PostSearchIndex},
        },
        i18n::{
            domain::locale::{EN_US_COUNTRY_CODE, EN_US_LANGUAGE_CODE},
            repository::i18n_repository::I18nRepository,
            service::i18n_service::I18nService,
        },
        reference_data::service::reference_data_service::CountryFlagLookupPort,
    },
    schema::i18n_strings,
};

use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::{account_test_context, seed_account},
};

struct EmptyCountryFlags;

#[async_trait::async_trait]
impl CountryFlagLookupPort for EmptyCountryFlags {
    async fn country_flag(&self, _country_code: i32) -> Option<String> { None }
    async fn country_flags(&self, _country_codes: &[i32]) -> HashMap<i32, String> {
        HashMap::new()
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit safe TEST_DATABASE_URL and PostgreSQL 18"]
async fn blog_cache_and_search_publish_committed_writes() -> TestResult {
    run_database_test(blog_cache_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit safe TEST_DATABASE_URL and PostgreSQL 18"]
async fn i18n_cache_refreshes_from_committed_rows() -> TestResult {
    run_database_test(i18n_cache_case).await
}

fn blog_cache_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let author = seed_account(&context, "BlogCacheAuthor").await?;
        context.accounts.verify_email(author.verification_token).await?;
        context.accounts.assign_role(author.user_id, RoleType::Younghyun).await?;
        let repository = Arc::new(BlogRepository::new(context.pool.clone()));
        let search = Arc::new(PostSearchIndex::new_in_memory()?);
        let flags: Arc<dyn CountryFlagLookupPort> = Arc::new(EmptyCountryFlags);
        let blog = BlogService::new(repository, search, flags);

        let created = blog.save_post(SavePostInput {
            actor_user_id: author.user_id,
            post_id: None,
            title: "Original cache sentinel".to_owned(),
            markdown: "A published cache-consistency body.".to_owned(),
            tags: vec!["cache-proof".to_owned()],
            published: true,
            owner_required: true,
        }).await?;
        let (cached, _) = blog.bounded_cache_page(1, 10, true).await;
        require(matches!(cached.first(), Some(post) if post.post_id == created.post_id),
            "committed post was not published to the metadata cache")?;
        let (search_hits, _) = blog.search_title("Original", 0, 10).await?;
        require(search_hits.iter().any(|post| post.post_id == created.post_id),
            "committed post was not published to search")?;

        blog.save_post(SavePostInput {
            actor_user_id: author.user_id,
            post_id: Some(created.post_id),
            title: "Replacement publication sentinel".to_owned(),
            markdown: "An updated cache-consistency body.".to_owned(),
            tags: vec!["replacement-proof".to_owned()],
            published: true,
            owner_required: true,
        }).await?;
        let (old_hits, _) = blog.search_title("Original", 0, 10).await?;
        let (new_hits, _) = blog.search_title("Replacement", 0, 10).await?;
        require(old_hits.is_empty() && new_hits.iter().any(|post| post.post_id == created.post_id),
            "post update left cache/search publication inconsistent")?;

        blog.delete_post(author.user_id, created.post_id).await?;
        let (cached, _) = blog.bounded_cache_page(1, 10, true).await;
        let (search_hits, _) = blog.search_title("Replacement", 0, 10).await?;
        require(cached.is_empty() && search_hits.is_empty(),
            "post deletion remained visible in cache or search")
    })
}

fn i18n_cache_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let pool = database.pool()?;
        let repository = Arc::new(I18nRepository::new(pool.clone()));
        let i18n = I18nService::new(repository);
        i18n.synchronize_file_sources().await?;
        i18n.synchronize_cache().await?;
        let before = i18n.ui_text_bundle(
            EN_US_COUNTRY_CODE, EN_US_LANGUAGE_CODE,
            EN_US_COUNTRY_CODE, EN_US_LANGUAGE_CODE, &["common.save"],
        ).await?;
        let before = before.get("common.save").cloned();

        let replacement = "Save cache refreshed";
        let mut connection = pool.get().await?;
        diesel::update(i18n_strings::table
            .filter(i18n_strings::i18n_string_reference_key.eq("common.save"))
            .filter(i18n_strings::i18n_string_country_code.eq(EN_US_COUNTRY_CODE))
            .filter(i18n_strings::i18n_string_language_code.eq(EN_US_LANGUAGE_CODE))
            .filter(i18n_strings::i18n_string_country_subdivision_code.is_null()))
            .set((i18n_strings::i18n_string_content.eq(replacement),
                i18n_strings::i18n_string_updated_at.eq(chrono::Utc::now())))
            .execute(&mut connection).await?;
        drop(connection);

        let stale = i18n.ui_text_bundle(
            EN_US_COUNTRY_CODE, EN_US_LANGUAGE_CODE,
            EN_US_COUNTRY_CODE, EN_US_LANGUAGE_CODE, &["common.save"],
        ).await?;
        require(stale.get("common.save") == before.as_ref(),
            "i18n cache changed before explicit synchronization")?;
        i18n.synchronize_cache().await?;
        let refreshed = i18n.ui_text_bundle(
            EN_US_COUNTRY_CODE, EN_US_LANGUAGE_CODE,
            EN_US_COUNTRY_CODE, EN_US_LANGUAGE_CODE, &["common.save"],
        ).await?;
        require(refreshed.get("common.save").is_some_and(|value| value == replacement),
            "i18n cache did not publish the committed database row")
    })
}
