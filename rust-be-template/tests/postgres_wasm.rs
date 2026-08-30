//! PostgreSQL coverage for deletion-aware WebAssembly metadata projection.

mod support;

use chrono::Utc;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use rust_be_template::{
    features::wasm::repository::wasm_repository::WasmRepository, schema::wasm_module,
};

use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::{VALID_PASSWORD, account_test_context, seed_account},
};

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn module_metadata_masks_a_deleted_owner() -> TestResult {
    run_database_test(deleted_owner_case).await
}

fn deleted_owner_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let owner = seed_account(&context, "WasmDeletedOwner").await?;
        context
            .accounts
            .verify_email(owner.verification_token)
            .await?;
        let module_id = Uuid::now_v7();
        let now = Utc::now();
        let mut connection = context.pool.get().await?;
        diesel::insert_into(wasm_module::table)
            .values((
                wasm_module::wasm_module_id.eq(module_id),
                wasm_module::user_id.eq(owner.user_id),
                wasm_module::wasm_module_link.eq(format!("/api/wasm-modules/{module_id}/wasm")),
                wasm_module::wasm_module_description.eq("retained module"),
                wasm_module::wasm_module_created_at.eq(now),
                wasm_module::wasm_module_updated_at.eq(now),
                wasm_module::wasm_module_thumbnail_link.eq("https://example.test/module.avif"),
                wasm_module::wasm_module_title.eq("Retained module"),
                wasm_module::wasm_module_bundle_gz.eq(vec![0_u8]),
            ))
            .execute(&mut connection)
            .await?;
        drop(connection);

        let repository = WasmRepository::new(context.pool.clone());
        let before = repository.list_metadata().await?;
        require(
            before.items.iter().any(|module| {
                module.wasm_module_id == module_id && module.user_id == owner.user_id
            }),
            "active WebAssembly owner was not projected",
        )?;

        context
            .accounts
            .soft_delete_account(owner.user_id, VALID_PASSWORD)
            .await?;
        let after = repository.list_metadata().await?;
        require(
            after
                .items
                .iter()
                .any(|module| module.wasm_module_id == module_id && module.user_id.is_nil()),
            "deleted WebAssembly owner retained a linkable public UUID",
        )
    })
}
