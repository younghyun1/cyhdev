use std::sync::atomic::{AtomicU64, AtomicUsize};

use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::bb8::Pool;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use std::sync::Arc;

use crate::domain::country::{CountryAndSubdivisionsTable, IsoCurrencyTable, IsoLanguageTable};
use crate::domain::i18n::i18n_cache::I18nCache;
use crate::domain::live_chat::cache::LiveChatCache;
use crate::domain::live_chat::rtc::{RtcConfig, RtcEngine};
use crate::features::accounts::{
    repository::account_repository::AccountRepository,
    service::{
        account_service::AccountService, auth_abuse::AuthAbuseService,
        session_service::SessionService,
    },
};
use crate::init::load_cache::fastfetch_cache::FastFetchCache;
use crate::init::load_cache::system_info::SystemInfoState;
use crate::init::load_cache::wasm_module_cache::WasmModuleCache;
use crate::init::search::PostSearchIndex;
use crate::util::geographic::ip_info_lookup::decompress_and_deserialize;
use tokio::sync::RwLock;
use tracing::{error, info};
use zeroize::Zeroizing;

use super::deployment_environment::DeploymentEnvironment;
use super::server_state::ServerState;
use super::server_state::blog_cache_policy::BlogCacheMetrics;

const DUMMY_PASSWORD: &str = "AuthTimingOnly4791";

#[derive(Default)]
pub struct ServerStateBuilder {
    app_name_version: Option<String>,
    server_start_time: Option<tokio::time::Instant>,
    pool: Option<Pool<AsyncPgConnection>>,
    email_client: Option<AsyncSmtpTransport<Tokio1Executor>>, // regexes: [regex::Regex; 1],
}

impl ServerStateBuilder {
    pub fn app_name_version(mut self, app_name_version: String) -> Self {
        self.app_name_version = Some(app_name_version);
        self
    }

    pub fn server_start_time(mut self, server_start_time: tokio::time::Instant) -> Self {
        self.server_start_time = Some(server_start_time);
        self
    }

    pub fn pool(mut self, pool: Pool<AsyncPgConnection>) -> Self {
        self.pool = Some(pool);
        self
    }

    pub fn email_client(mut self, email_client: AsyncSmtpTransport<Tokio1Executor>) -> Self {
        self.email_client = Some(email_client);
        self
    }

    pub async fn build(self) -> anyhow::Result<ServerState> {
        let deployment_environment = DeploymentEnvironment::from_env()?;
        let pool = self
            .pool
            .ok_or_else(|| anyhow::anyhow!("pool is required"))?;
        let email_client = self
            .email_client
            .ok_or_else(|| anyhow::anyhow!("email_client is required"))?;
        let account_repository = Arc::new(AccountRepository::new(pool.clone()));
        let auth_abuse_service = Arc::new(AuthAbuseService::new().map_err(|error| {
            anyhow::anyhow!("operating-system entropy unavailable for auth limiter: {error}")
        })?);
        let session_service = Arc::new(SessionService::new());
        let live_chat_cache = Arc::new(LiveChatCache::default());
        let dummy_password_hash = crate::util::crypto::hash_pw::hash_pw(Zeroizing::new(
            DUMMY_PASSWORD.to_owned(),
        ))
        .await
        .map_err(|error| anyhow::anyhow!("failed to initialize dummy password hash: {error}"))?;
        let account_service = Arc::new(AccountService::new(
            account_repository,
            Arc::clone(&session_service),
            Arc::clone(&live_chat_cache),
            email_client,
            dummy_password_hash,
        ));

        let aws_profile_picture_config = {
            use aws_config::BehaviorVersion;
            use aws_config::meta::region::RegionProviderChain;

            let aws_key = std::env::var("AWS_IMAGE_UPLOAD_KEY")
                .map_err(|_| anyhow::anyhow!("AWS_IMAGE_UPLOAD_KEY not set"))?;
            let aws_secret = std::env::var("AWS_IMAGE_UPLOAD_SECRET_KEY")
                .map_err(|_| anyhow::anyhow!("AWS_IMAGE_UPLOAD_SECRET_KEY not set"))?;
            let credentials = aws_sdk_s3::config::Credentials::new(
                aws_key,
                aws_secret,
                None,                     // token
                None,                     // expiration
                "cyhdev-profile-picture", // provider name
            );
            // Use default region chain or fallback if not set.
            let region_provider = RegionProviderChain::default_provider().or_else("us-west-1");
            aws_config::defaults(BehaviorVersion::latest())
                .region(region_provider)
                .credentials_provider(credentials)
                .load()
                .await
        };

        let fastfetch_cache = FastFetchCache::init().await;

        // Build the SFU engine once if enabled. A bind/init failure disables RTC
        // but does not abort startup.
        let rtc_config = RtcConfig::from_env();
        let rtc_engine = if rtc_config.enabled {
            match RtcEngine::new(rtc_config.clone()).await {
                Ok(engine) => Some(Arc::new(engine)),
                Err(e) => {
                    error!(error = %e, "Failed to initialize RTC SFU engine; calls disabled");
                    None
                }
            }
        } else {
            info!("RTC SFU disabled (RTC_ENABLE not set)");
            None
        };

        Ok(ServerState {
            app_name_version: self
                .app_name_version
                .ok_or_else(|| anyhow::anyhow!("app_name_version is required"))?,
            server_start_time: self
                .server_start_time
                .ok_or_else(|| anyhow::anyhow!("server_start_time is required"))?,
            pool,
            responses_handled: AtomicU64::new(0u64),
            account_service,
            auth_abuse_service,
            session_service,
            blog_posts_cache: scc::HashMap::new(),
            blog_post_slug_cache: scc::HashMap::new(),
            blog_post_order_cache: RwLock::new(Vec::new()),
            blog_cache_mutation: tokio::sync::Mutex::new(()),
            blog_cache_metrics: BlogCacheMetrics::default(),
            search_index: {
                // Use disk-persisted index, configurable via env var
                let index_path = std::env::var("SEARCH_INDEX_PATH")
                    .unwrap_or_else(|_| "./data/search_index".to_string());
                let index = PostSearchIndex::open_or_create(&index_path)?;
                info!(path = %index_path, "Search index initialized");
                index
            },
            geo_ip_db: {
                let (dbs, dur) = decompress_and_deserialize()?;
                info!(elapsed=%format!("{dur:?}"), "Geo-IP database loaded and interned.");
                dbs
            },
            country_map: RwLock::new(CountryAndSubdivisionsTable::new_empty()),
            languages_map: RwLock::new(IsoLanguageTable::new_empty()),
            currency_map: RwLock::new(IsoCurrencyTable::new_empty()),
            i18n_cache: RwLock::new(I18nCache::new()),
            deployment_environment,
            request_client: reqwest::Client::builder()
                .user_agent("cyhdev.com")
                .build()?,
            visitor_board_map: scc::HashMap::new(),
            visitor_board_entry_count: AtomicUsize::new(0),
            visitor_board_rejected_entries: AtomicU64::new(0),
            visitor_log_buffer: scc::HashMap::new(),
            visitor_log_entry_count: AtomicUsize::new(0),
            visitor_log_pending_events: AtomicUsize::new(0),
            visitor_log_rejected_admissions: AtomicU64::new(0),
            system_info_state: SystemInfoState::new(),
            aws_profile_picture_config,
            fastfetch: fastfetch_cache,
            wasm_module_cache: WasmModuleCache::default(),
            live_chat_cache,
            rtc_config,
            rtc_engine,
            rtc_rooms: scc::HashMap::new(),
            photograph_batches: scc::HashMap::new(),
            photograph_batch_count: AtomicUsize::new(0),
            photograph_view_buffer: tokio::sync::RwLock::new(std::collections::HashMap::new()),
            photograph_view_rejected_events: AtomicU64::new(0),
        })
    }
}
