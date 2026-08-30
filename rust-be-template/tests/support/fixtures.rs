//! Account fixtures built through public repository and service boundaries.

use std::{path::PathBuf, sync::Arc};

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::bb8::Pool};
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use uuid::Uuid;
use zeroize::Zeroizing;

use rust_be_template::{
    features::live_chat::service::cache::LiveChatCache,
    features::accounts::{
        domain::account::SignupCommand,
        repository::account_repository::AccountRepository,
        service::{account_service::{AccountService, AccountServiceDependencies}, session_service::SessionService},
    },
    schema::{email_verification_tokens, iso_country},
    util::media::object_store::{
        MediaObjectStore, MediaObjectStoreFuture, ObjectLocation,
    },
};

use super::database::{HarnessError, TestDatabase, TestResult};

pub const VALID_PASSWORD: &str = "ValidPass123";
const TEST_DUMMY_PASSWORD_HASH: &str = "$argon2id$v=19$m=256,t=2,p=1$c29tZXNhbHQ$nf65EOgLrQMR/uIPnA4rEsF5h7TKyQwu9U1bMCHGi/4";

struct TestMediaObjectStore;

impl MediaObjectStore for TestMediaObjectStore {
    fn put_file<'a>(
        &'a self,
        _location: ObjectLocation,
        _content_type: String,
        _source: PathBuf,
    ) -> MediaObjectStoreFuture<'a> {
        Box::pin(async { Ok(()) })
    }

    fn delete<'a>(&'a self, _location: ObjectLocation) -> MediaObjectStoreFuture<'a> {
        Box::pin(async { Ok(()) })
    }
}

pub struct AccountTestContext {
    pub accounts: Arc<AccountService>,
    pub live_chat_cache: Arc<LiveChatCache>,
    pub repository: Arc<AccountRepository>,
    pub sessions: Arc<SessionService>,
    pub pool: Pool<AsyncPgConnection>,
}

pub struct AccountFixture {
    pub user_id: Uuid,
    pub user_name: String,
    pub email: String,
    pub verification_token: Uuid,
    pub country: i32,
    pub language: i32,
}

pub fn account_test_context(database: &TestDatabase) -> TestResult<AccountTestContext> {
    let pool = database.pool()?;
    let repository = Arc::new(AccountRepository::new(pool.clone()));
    let sessions = Arc::new(SessionService::new());
    let live_chat_cache = Arc::new(LiveChatCache::default());
    let live_chat_lifecycle: Arc<
        dyn rust_be_template::features::live_chat::service::lifecycle::LiveChatAccountLifecyclePort,
    > = live_chat_cache.clone();
    let email_client = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("127.0.0.1")
        .port(9)
        .build();
    let accounts = Arc::new(AccountService::new(AccountServiceDependencies {
        repository: Arc::clone(&repository),
        sessions: Arc::clone(&sessions),
        live_chat_lifecycle,
        media_object_store: Arc::new(TestMediaObjectStore),
        media_region: Arc::from("us-west-1"),
        email_client,
        public_app_origin: Arc::from("https://app.example.test"),
        dummy_password_hash: TEST_DUMMY_PASSWORD_HASH.to_owned(),
    }));

    Ok(AccountTestContext {
        accounts,
        live_chat_cache,
        repository,
        sessions,
        pool,
    })
}

pub async fn seed_account(context: &AccountTestContext, label: &str) -> TestResult<AccountFixture> {
    let mut connection = context.pool.get().await?;
    let (country, language) = iso_country::table
        .filter(iso_country::is_country.eq(true))
        .order(iso_country::country_code.asc())
        .select((
            iso_country::country_code,
            iso_country::country_primary_language,
        ))
        .first::<(i32, i32)>(&mut connection)
        .await?;
    drop(connection);

    let user_name = label.to_owned();
    let email = format!("{}@example.test", label.to_ascii_lowercase());
    context
        .accounts
        .signup(SignupCommand {
            user_name: user_name.clone(),
            user_email: email.clone(),
            password: Zeroizing::new(VALID_PASSWORD.to_owned()),
            country,
            language,
            subdivision: None,
        })
        .await?;

    let account = match context.repository.login_account_by_email(&email).await? {
        Some(account) => account,
        None => {
            return Err(Box::new(HarnessError::Assertion {
                message: "registered account was not readable",
            }));
        }
    };
    let mut connection = context.pool.get().await?;
    let verification_token = email_verification_tokens::table
        .filter(email_verification_tokens::user_id.eq(account.user_id))
        .select(email_verification_tokens::email_verification_token)
        .first::<Uuid>(&mut connection)
        .await?;
    drop(connection);

    Ok(AccountFixture {
        user_id: account.user_id,
        user_name,
        email,
        verification_token,
        country,
        language,
    })
}
