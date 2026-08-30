//! Imports kept separate from root router composition.

pub(super) use std::sync::Arc;

pub(super) use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{Method, header},
    middleware::{from_fn, from_fn_with_state},
    routing::{delete, get, patch, post},
};
pub(super) use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
};
pub(super) use utoipa::OpenApi;
pub(super) use utoipa_swagger_ui::SwaggerUi;

pub(super) use crate::{
    docs::ApiDoc,
    features::accounts::api::{
        auth_abuse::{enforce_auth_ip_throttle, sensitive_auth_response_headers},
        authorization_audit::list_authorization_audit,
        authorization_mutations::{
            assign_authorization_role, set_authorization_role_permission,
        },
        authorization_queries::{
            list_authorization_permissions, list_authorization_roles,
            list_authorization_users, list_role_permissions,
        },
        delete_account::delete_account,
        hard_purge_account::hard_purge_account, is_superuser::is_superuser_handler, login::login,
        logout::logout, me::me_handler,
        media_cleanup::{resolve_media_cleanup, unresolved_media_cleanup},
        oidc_callback::oidc_callback,
        oidc_link::{complete_oidc_link, unlink_oidc},
        oidc_start::{start_oidc_link, start_oidc_login},
        oidc_status::oidc_status,
        profile_picture_history::{
            delete_profile_picture, list_profile_pictures, select_profile_picture,
        },
        public_user::get_user_info,
        reset_password::reset_password,
        reset_password_request::reset_password_request_process,
        retention_notifications::{retention_notification_status, retry_retention_notification},
        signup::signup_handler,
        update_profile::update_profile,
        upload_profile_picture::upload_profile_picture,
        verify_user_email::verify_user_email,
    },
    features::blog::api::{
        delete_comment::delete_comment, delete_post::delete_post, get_posts::get_posts,
        read_post::read_post, rescind_comment_vote::rescind_comment_vote,
        rescind_post_vote::rescind_post_vote, search_posts::search_posts,
        submit_comment::submit_comment, submit_post::submit_post,
        update_comment::update_comment, update_post::update_post,
        vote_comment::vote_comment, vote_post::vote_post,
    },
    features::forum::api::{
        audit::list_forum_moderation_audit,
        capabilities::forum_capabilities,
        notifications::{list_forum_notifications, mark_forum_notification_read},
        replies::{
            create_forum_reply, delete_forum_reply, moderate_forum_reply, update_forum_reply,
        },
        subscriptions::{subscribe_forum_topic, unsubscribe_forum_topic},
        topics::{
            create_forum_topic, delete_forum_topic, get_forum_topic, list_forum_topics,
            moderate_forum_topic, update_forum_topic,
        },
    },
    features::geo::api::lookup::{lookup_ip_info, lookup_ip_location, lookup_my_ip_info},
    features::i18n::api::{
        get_ui_text_bundle::get_ui_text_bundle,
        sync_i18n_cache::sync_i18n_cache,
    },
    features::live_chat::api::{
        cache_stats::get_live_chat_cache_stats,
        get_messages::get_live_chat_messages,
        ws::live_chat_ws_handler,
    },
    features::photography::api::{
        batch_list::batch_list, batch_status::batch_status, batch_upload::batch_upload,
        delete_photograph_comment::delete_photograph_comment,
        delete_photographs::delete_photographs, get_photographs::get_photographs,
        read_photograph::read_photograph,
        rescind_photograph_comment_vote::rescind_photograph_comment_vote,
        rescind_photograph_vote::rescind_photograph_vote,
        submit_photograph_comment::submit_photograph_comment,
        update_photograph_comment::update_photograph_comment,
        upload_photograph::upload_photograph, vote_photograph::vote_photograph,
        vote_photograph_comment::vote_photograph_comment,
    },
    features::reference_data::api::{
        get_countries::get_countries, get_country::get_country,
        get_language::get_language, get_languages::get_languages,
        get_subdivisions::get_subdivisions_for_country,
    },
    features::server_status::api::{
        host_stats::ws_host_stats_handler,
        http::{get_host_fastfetch, healthcheck, root_handler},
    },
    features::visitor::api::visitor_board::get_visitor_board_entries,
    features::wasm::api::{
        delete_module::delete_wasm_module, list_modules::get_wasm_modules,
        serve_bundle::serve_wasm, update_assets::update_wasm_module_assets,
        update_metadata::update_wasm_module, upload_module::upload_wasm_module,
    },
    init::state::{DeploymentEnvironment, ServerState},
};

pub(super) use super::middleware::{
    auth::auth_middleware, is_logged_in::is_logged_in_middleware, logging::log_middleware,
    role::require_superuser_middleware,
    trusted_origin::{TrustedOrigins, require_trusted_origin},
};
