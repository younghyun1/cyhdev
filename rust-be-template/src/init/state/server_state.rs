use std::sync::Arc;

use crate::features::accounts::service::{
    account_service::AccountService, auth_abuse::AuthAbuseService, oidc::provider::OidcService,
    session_service::SessionService,
};
use crate::features::blog::service::blog_service::BlogService;
use crate::features::forum::service::forum_service::ForumService;
use crate::features::geo::service::geo_service::GeoService;
use crate::features::i18n::service::i18n_service::I18nService;
use crate::features::live_chat::service::live_chat_service::LiveChatService;
use crate::features::photography::service::photography_service::PhotographyService;
use crate::features::reference_data::service::reference_data_service::ReferenceDataService;
use crate::features::server_status::service::server_status_service::ServerStatusService;
use crate::features::visitor::service::visitor_service::VisitorService;
use crate::features::wasm::service::wasm_service::WasmService;

use super::{deployment_environment::DeploymentEnvironment, public_app_origin::PublicAppOrigin};
mod core;

pub struct ServerState {
    pub(crate) account_service: Arc<AccountService>,
    pub(crate) blog_service: Arc<BlogService>,
    pub(crate) forum_service: Arc<ForumService>,
    pub(crate) geo_service: Arc<GeoService>,
    pub(crate) i18n_service: Arc<I18nService>,
    pub(crate) live_chat_service: Arc<LiveChatService>,
    pub(crate) photography_service: Arc<PhotographyService>,
    pub(crate) reference_data_service: Arc<ReferenceDataService>,
    pub(crate) server_status_service: Arc<ServerStatusService>,
    pub(crate) visitor_service: Arc<VisitorService>,
    pub(crate) wasm_service: Arc<WasmService>,
    pub(crate) auth_abuse_service: Arc<AuthAbuseService>,
    pub(crate) oidc_service: Arc<OidcService>,
    pub(crate) session_service: Arc<SessionService>,
    pub(crate) deployment_environment: DeploymentEnvironment,
    pub(crate) public_app_origin: PublicAppOrigin,
}
