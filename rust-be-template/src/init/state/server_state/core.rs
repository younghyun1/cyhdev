use std::sync::Arc;
use super::ServerState;
use crate::features::accounts::service::{
    account_service::AccountService, auth_abuse::AuthAbuseService,
    oidc::provider::OidcService,
    session_service::SessionService,
};
use crate::features::forum::service::forum_service::ForumService;
use crate::features::geo::service::geo_service::GeoService;
use crate::features::blog::service::blog_service::BlogService;
use crate::features::i18n::service::i18n_service::I18nService;
use crate::features::live_chat::service::{
    live_chat_service::LiveChatService, rtc::coordinator::RtcCoordinator,
};
use crate::features::photography::service::photography_service::PhotographyService;
use crate::features::reference_data::service::reference_data_service::ReferenceDataService;
use crate::features::server_status::service::server_status_service::ServerStatusService;
use crate::features::visitor::service::visitor_service::VisitorService;
use crate::features::wasm::service::wasm_service::WasmService;
use crate::init::state::{DeploymentEnvironment, PublicAppOrigin, ServerStateBuilder};

impl ServerState {
    pub fn builder() -> ServerStateBuilder {
        ServerStateBuilder::default()
    }

    pub fn account_service(&self) -> Arc<AccountService> {
        Arc::clone(&self.account_service)
    }

    pub fn blog_service(&self) -> Arc<BlogService> {
        Arc::clone(&self.blog_service)
    }

    pub fn forum_service(&self) -> Arc<ForumService> {
        Arc::clone(&self.forum_service)
    }

    pub fn geo_service(&self) -> Arc<GeoService> {
        Arc::clone(&self.geo_service)
    }

    pub fn i18n_service(&self) -> Arc<I18nService> {
        Arc::clone(&self.i18n_service)
    }

    pub fn live_chat_service(&self) -> Arc<LiveChatService> {
        Arc::clone(&self.live_chat_service)
    }

    pub fn rtc_service(&self) -> Arc<RtcCoordinator> {
        self.live_chat_service.rtc()
    }

    pub fn photography_service(&self) -> Arc<PhotographyService> {
        Arc::clone(&self.photography_service)
    }

    pub fn reference_data_service(&self) -> Arc<ReferenceDataService> {
        Arc::clone(&self.reference_data_service)
    }

    pub fn server_status_service(&self) -> Arc<ServerStatusService> {
        Arc::clone(&self.server_status_service)
    }

    pub fn visitor_service(&self) -> Arc<VisitorService> {
        Arc::clone(&self.visitor_service)
    }

    pub fn wasm_service(&self) -> Arc<WasmService> {
        Arc::clone(&self.wasm_service)
    }

    pub fn auth_abuse_service(&self) -> Arc<AuthAbuseService> {
        Arc::clone(&self.auth_abuse_service)
    }

    pub fn oidc_service(&self) -> Arc<OidcService> {
        Arc::clone(&self.oidc_service)
    }

    pub fn session_service(&self) -> Arc<SessionService> {
        Arc::clone(&self.session_service)
    }

    pub fn get_deployment_environment(&self) -> DeploymentEnvironment {
        self.deployment_environment
    }

    pub fn public_app_origin(&self) -> PublicAppOrigin {
        self.public_app_origin.clone()
    }

}
