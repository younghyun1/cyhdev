use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::bb8::Pool;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use std::sync::Arc;

use crate::features::accounts::{
    repository::account_repository::AccountRepository,
    service::{
        account_service::{AccountService, AccountServiceDependencies},
        auth_abuse::AuthAbuseService,
        oidc::provider::OidcService,
        session_service::SessionService,
    },
};
use crate::features::blog::{
    repository::blog_repository::BlogRepository,
    service::{blog_service::BlogService, search::search_index::PostSearchIndex},
};
use crate::features::forum::{
    repository::forum_repository::ForumRepository, service::forum_service::ForumService,
};
use crate::features::geo::{
    repository::{geo_ip_database::GeoIpDatabases, geo_repository::GeoRepository},
    service::geo_service::{GeoCountryFlagPort, GeoService},
};
use crate::features::i18n::{
    repository::i18n_repository::I18nRepository, service::i18n_service::I18nService,
};
use crate::features::live_chat::{
    repository::live_chat_repository::LiveChatRepository,
    service::{
        cache::LiveChatCache,
        lifecycle::LiveChatAccountLifecyclePort,
        live_chat_service::LiveChatService,
        ports::{CountryAlpha2FlagPort, GeoIpLookupPort, ReferenceDataAlpha2Flags},
        rtc::{config::RtcConfig, coordinator::RtcCoordinator, engine::RtcEngine},
    },
};
use crate::features::photography::{
    repository::photography_repository::PhotographyRepository,
    service::photography_service::PhotographyService,
};
use crate::features::reference_data::{
    repository::reference_data_repository::ReferenceDataRepository,
    service::reference_data_service::ReferenceDataService,
};
use crate::features::server_status::{
    repository::server_status_repository::ServerStatusRepository,
    service::server_status_service::ServerStatusService,
};
use crate::features::visitor::{
    repository::visitor_repository::VisitorRepository, service::visitor_service::VisitorService,
};
use crate::features::wasm::{
    repository::wasm_repository::WasmRepository,
    service::{cache::WasmModuleCache, wasm_service::WasmService},
};
use crate::util::media::object_store::{MediaObjectStore, S3MediaObjectStore};
use tracing::{error, info};
use zeroize::Zeroizing;

use super::server_state::ServerState;
use super::{deployment_environment::DeploymentEnvironment, public_app_origin::PublicAppOrigin};

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
        let public_app_origin = PublicAppOrigin::from_environment(deployment_environment)?;
        let pool = self
            .pool
            .ok_or_else(|| anyhow::anyhow!("pool is required"))?;
        let email_client = self
            .email_client
            .ok_or_else(|| anyhow::anyhow!("email_client is required"))?;
        let app_name_version = self
            .app_name_version
            .ok_or_else(|| anyhow::anyhow!("app_name_version is required"))?;
        let server_start_time = self
            .server_start_time
            .ok_or_else(|| anyhow::anyhow!("server_start_time is required"))?;
        let media_config = {
            use aws_config::BehaviorVersion;
            use aws_config::meta::region::RegionProviderChain;

            let aws_key = std::env::var("AWS_IMAGE_UPLOAD_KEY")
                .map_err(|_| anyhow::anyhow!("AWS_IMAGE_UPLOAD_KEY not set"))?;
            let aws_secret = std::env::var("AWS_IMAGE_UPLOAD_SECRET_KEY")
                .map_err(|_| anyhow::anyhow!("AWS_IMAGE_UPLOAD_SECRET_KEY not set"))?;
            let credentials = aws_sdk_s3::config::Credentials::new(
                aws_key,
                aws_secret,
                None,
                None,
                "cyhdev-media",
            );
            let region_provider = RegionProviderChain::default_provider().or_else("us-west-1");
            aws_config::defaults(BehaviorVersion::latest())
                .region(region_provider)
                .credentials_provider(credentials)
                .load()
                .await
        };
        let media_region: Arc<str> = Arc::from(
            media_config
                .region()
                .map(ToString::to_string)
                .unwrap_or_else(|| "us-west-1".to_owned()),
        );
        let media_object_store: Arc<dyn MediaObjectStore> =
            Arc::new(S3MediaObjectStore::from_config(&media_config));
        let account_repository = Arc::new(AccountRepository::new(pool.clone()));
        let auth_abuse_service = Arc::new(AuthAbuseService::new().map_err(|error| {
            anyhow::anyhow!("operating-system entropy unavailable for auth limiter: {error}")
        })?);
        let session_service = Arc::new(SessionService::new());
        let oidc_service = Arc::new(
            OidcService::from_environment(deployment_environment, &public_app_origin).await?,
        );
        let live_chat_cache = Arc::new(LiveChatCache::default());
        let live_chat_lifecycle: Arc<dyn LiveChatAccountLifecyclePort> = live_chat_cache.clone();
        let dummy_password_hash =
            crate::util::crypto::hash_pw::hash_pw(Zeroizing::new(DUMMY_PASSWORD.to_owned()))
                .await
                .map_err(|error| {
                    anyhow::anyhow!("failed to initialize dummy password hash: {error}")
                })?;
        let account_service = Arc::new(AccountService::new(AccountServiceDependencies {
            repository: account_repository,
            sessions: Arc::clone(&session_service),
            live_chat_lifecycle,
            media_object_store: Arc::clone(&media_object_store),
            media_region: Arc::clone(&media_region),
            email_client,
            public_app_origin: public_app_origin.as_arc(),
            dummy_password_hash,
        }));
        let forum_repository = Arc::new(ForumRepository::new(pool.clone()));
        let forum_service = Arc::new(ForumService::new(
            forum_repository,
            Arc::clone(&account_service),
        ));
        let i18n_service = Arc::new(I18nService::new(Arc::new(I18nRepository::new(
            pool.clone(),
        ))));
        let reference_data_service = Arc::new(ReferenceDataService::new(Arc::new(
            ReferenceDataRepository::new(pool.clone()),
        )));
        let search_index_path =
            std::env::var("SEARCH_INDEX_PATH").unwrap_or_else(|_| "./data/search_index".to_owned());
        let post_search_index = Arc::new(PostSearchIndex::open_or_create(&search_index_path)?);
        info!(path = %search_index_path, "Search index initialized");
        let blog_country_flags: Arc<
            dyn crate::features::reference_data::service::reference_data_service::CountryFlagLookupPort,
        > = reference_data_service.clone();
        let blog_service = Arc::new(BlogService::new(
            Arc::new(BlogRepository::new(pool.clone())),
            post_search_index,
            blog_country_flags,
        ));
        let (geo_databases, geo_load_duration) = GeoIpDatabases::load_default()?;
        info!(elapsed = ?geo_load_duration, "Geo-IP databases loaded");
        let geo_country_flags: Arc<dyn GeoCountryFlagPort> = reference_data_service.clone();
        let geo_service = Arc::new(GeoService::new(
            geo_databases,
            geo_country_flags,
            Arc::new(GeoRepository::new(pool.clone())),
        ));
        let visitor_service = Arc::new(VisitorService::new(
            Arc::new(VisitorRepository::new(pool.clone())),
            Arc::clone(&geo_service),
        ));
        let server_status_service = Arc::new(ServerStatusService::new(
            Arc::new(ServerStatusRepository::new(pool.clone())),
            app_name_version,
            server_start_time,
        ));
        server_status_service.initialize().await;

        let wasm_service = Arc::new(WasmService::new(
            Arc::new(WasmRepository::new(pool.clone())),
            WasmModuleCache::default(),
            Arc::clone(&media_object_store),
            Arc::clone(&media_region),
            Arc::clone(&account_service),
        ));
        let photography_country_flags: Arc<
            dyn crate::features::reference_data::service::reference_data_service::CountryFlagLookupPort,
        > = reference_data_service.clone();
        let photography_service = Arc::new(PhotographyService::new(
            PhotographyRepository::new(pool.clone()),
            media_object_store,
            media_region,
            Arc::clone(&account_service),
            photography_country_flags,
        ));

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
        let live_chat_repository = Arc::new(LiveChatRepository::new(pool.clone()));
        let rtc_service = Arc::new(RtcCoordinator::new(
            rtc_engine,
            rtc_config.max_participants,
            Arc::clone(&live_chat_cache),
            Arc::clone(&live_chat_repository),
        ));
        let live_chat_country_flags: Arc<
            dyn crate::features::reference_data::service::reference_data_service::CountryFlagLookupPort,
        > = reference_data_service.clone();
        let live_chat_alpha2_flags: Arc<dyn CountryAlpha2FlagPort> =
            Arc::new(ReferenceDataAlpha2Flags {
                reference_data: Arc::clone(&reference_data_service),
            });
        let live_chat_geo_ip: Arc<dyn GeoIpLookupPort> = geo_service.clone();
        let live_chat_service = Arc::new(LiveChatService::new(
            live_chat_repository,
            live_chat_cache,
            live_chat_country_flags,
            live_chat_alpha2_flags,
            live_chat_geo_ip,
            rtc_service,
        ));

        Ok(ServerState {
            account_service,
            blog_service,
            forum_service,
            geo_service,
            i18n_service,
            live_chat_service,
            photography_service,
            reference_data_service,
            server_status_service,
            visitor_service,
            wasm_service,
            auth_abuse_service,
            oidc_service,
            session_service,
            deployment_environment,
            public_app_origin,
        })
    }
}
