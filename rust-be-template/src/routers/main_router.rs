use super::main_router_registry::*;

mod static_assets;
use static_assets::static_asset_handler;

const MAX_REQUEST_SIZE: usize = 1024 * 1024 * 150; // 150MB
const BATCH_REQUEST_SIZE: usize = 1024 * 1024 * 1024; // 1GB (route-scoped to batch upload)
const AUTH_REQUEST_SIZE: usize = 8 * 1024;
const FORUM_REQUEST_SIZE: usize = 128 * 1024;

pub fn build_router(state: Arc<ServerState>) -> anyhow::Result<axum::Router> {
    let sessions = state.session_service();
    let auth_middleware = from_fn_with_state(Arc::clone(&sessions), auth_middleware);
    let require_superuser_middleware = from_fn(require_superuser_middleware);
    let log_middleware = from_fn_with_state(state.clone(), log_middleware);
    let is_logged_in_middleware = from_fn_with_state(sessions, is_logged_in_middleware);
    let trusted_origins = Arc::new(TrustedOrigins::from_environment(
        state.get_deployment_environment(),
        &state.public_app_origin(),
    )?);
    let trusted_origin_middleware =
        from_fn_with_state(Arc::clone(&trusted_origins), require_trusted_origin);
    let compression_middleware = CompressionLayer::new().zstd(true).gzip(true);

    let cors_layer = CorsLayer::new()
        .allow_origin(AllowOrigin::list(
            trusted_origins.header_values().iter().cloned(),
        ))
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .expose_headers([
            header::HeaderName::from_static("x-server-built-time"),
            header::HeaderName::from_static("x-server-name"),
            header::HeaderName::from_static("x-server-rust-version"),
            header::RETRY_AFTER,
        ]);

    let public_router = Router::new()
        .route("/api/healthcheck/server", get(healthcheck))
        .route("/api/healthcheck/state", get(root_handler))
        .route("/api/healthcheck/fastfetch", get(get_host_fastfetch))
        .route("/ws/host-stats", get(ws_host_stats_handler))
        .route("/ws/live-chat", get(live_chat_ws_handler))
        .route("/api/dropdown/language", get(get_languages))
        .route("/api/dropdown/language/{language_id}", get(get_language))
        .route("/api/dropdown/country", get(get_countries))
        .route("/api/dropdown/country/{country_id}", get(get_country))
        .route(
            "/api/dropdown/country/{country_id}/subdivision",
            get(get_subdivisions_for_country),
        )
        .route("/api/visitor-board", get(get_visitor_board_entries))
        .route("/api/geolocate/{ip_address}", get(lookup_ip_location))
        .route("/api/geo-ip-info/me", get(lookup_my_ip_info))
        .route("/api/geo-ip-info/{ip_address}", get(lookup_ip_info))
        .route("/api/auth/me", get(me_handler))
        .route("/api/auth/oidc/status", get(oidc_status))
        .route("/api/forum/capabilities", get(forum_capabilities))
        .route("/api/forum/topics", get(list_forum_topics))
        .route("/api/forum/topics/{topic_id}", get(get_forum_topic))
        .route("/api/auth/is-superuser", get(is_superuser_handler))
        .route("/api/users/{user_name}", get(get_user_info))
        .route("/api/blog/posts", get(get_posts))
        .route("/api/blog/posts/{post_id}", get(read_post))
        .route("/api/blog/search", get(search_posts))
        .route("/api/live-chat/messages", get(get_live_chat_messages))
        .route("/api/live-chat/cache-stats", get(get_live_chat_cache_stats))
        .route("/api/i18n/ui-text", get(get_ui_text_bundle))
        .route("/api/photographs/get", get(get_photographs))
        .route("/api/photographs/{photograph_id}", get(read_photograph))
        .route("/api/wasm-modules", get(get_wasm_modules))
        .route("/api/wasm-modules/{wasm_module_id}/wasm", get(serve_wasm));

    let auth_abuse_router = Router::new()
        .route("/api/auth/signup", post(signup_handler))
        .route("/api/auth/login", post(login))
        .route("/api/auth/oidc/login/start", post(start_oidc_login))
        .route(
            "/api/auth/reset-password-request",
            post(reset_password_request_process),
        )
        .route("/api/auth/reset-password", post(reset_password))
        .route("/api/auth/verify-user-email", post(verify_user_email))
        .layer(DefaultBodyLimit::max(AUTH_REQUEST_SIZE))
        .layer(from_fn_with_state(
            state.auth_abuse_service(),
            enforce_auth_ip_throttle,
        ))
        .layer(from_fn(sensitive_auth_response_headers));

    let oidc_callback_router = Router::new()
        .route("/api/auth/oidc/callback", get(oidc_callback))
        .layer(from_fn(sensitive_auth_response_headers));

    let protected_oidc_router = Router::new()
        .route("/api/auth/oidc/link/start", post(start_oidc_link))
        .route("/api/auth/oidc/link/complete", post(complete_oidc_link))
        .route("/api/auth/oidc/link", delete(unlink_oidc))
        .layer(DefaultBodyLimit::max(AUTH_REQUEST_SIZE))
        .layer(auth_middleware.clone())
        .layer(from_fn_with_state(
            state.auth_abuse_service(),
            enforce_auth_ip_throttle,
        ))
        .layer(from_fn(sensitive_auth_response_headers));

    let protected_forum_router = Router::new()
        .route("/api/forum/topics", post(create_forum_topic))
        .route("/api/forum/topics/{topic_id}", patch(update_forum_topic).delete(delete_forum_topic))
        .route("/api/forum/topics/{topic_id}/replies", post(create_forum_reply))
        .route("/api/forum/replies/{reply_id}", patch(update_forum_reply).delete(delete_forum_reply))
        .route("/api/forum/topics/{topic_id}/subscription", post(subscribe_forum_topic).delete(unsubscribe_forum_topic))
        .route("/api/forum/topics/{topic_id}/moderation", post(moderate_forum_topic))
        .route("/api/forum/replies/{reply_id}/moderation", post(moderate_forum_reply))
        .route("/api/forum/moderation/audit", get(list_forum_moderation_audit))
        .route("/api/forum/notifications", get(list_forum_notifications))
        .route("/api/forum/notifications/{notification_id}/read", post(mark_forum_notification_read))
        .layer(DefaultBodyLimit::max(FORUM_REQUEST_SIZE))
        .layer(auth_middleware.clone());

    let protected_account_router = Router::new()
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/account", delete(delete_account))
        .route("/api/auth/profile", patch(update_profile))
        .layer(DefaultBodyLimit::max(AUTH_REQUEST_SIZE));

    let protected_router = Router::new()
        .route(
            "/api/user/upload-profile-picture",
            post(upload_profile_picture),
        )
        .route("/api/user/profile-pictures", get(list_profile_pictures))
        .route(
            "/api/user/profile-pictures/{profile_picture_id}/select",
            post(select_profile_picture),
        )
        .route(
            "/api/user/profile-pictures/{profile_picture_id}",
            delete(delete_profile_picture),
        )
        .route("/api/blog/{post_id}/vote", post(vote_post))
        .route("/api/blog/{post_id}/{comment_id}/vote", post(vote_comment))
        .route("/api/blog/{post_id}/vote", delete(rescind_post_vote))
        .route("/api/blog/{post_id}/{comment_id}", delete(delete_comment))
        .route("/api/blog/{post_id}/{comment_id}", patch(update_comment))
        .route("/api/blog/{post_id}", delete(delete_post))
        .route("/api/blog/{post_id}/comment", post(submit_comment))
        .route(
            "/api/blog/{post_id}/{comment_id}/vote",
            delete(rescind_comment_vote),
        )
        .route(
            "/api/photographs/{photograph_id}/vote",
            post(vote_photograph),
        )
        .route(
            "/api/photographs/{photograph_id}/vote",
            delete(rescind_photograph_vote),
        )
        .route(
            "/api/photographs/{photograph_id}/comment",
            post(submit_photograph_comment),
        )
        .route(
            "/api/photographs/{photograph_id}/{comment_id}/vote",
            post(vote_photograph_comment),
        )
        .route(
            "/api/photographs/{photograph_id}/{comment_id}/vote",
            delete(rescind_photograph_comment_vote),
        )
        .route(
            "/api/photographs/{photograph_id}/{comment_id}",
            patch(update_photograph_comment),
        )
        .route(
            "/api/photographs/{photograph_id}/{comment_id}",
            delete(delete_photograph_comment),
        )
        .merge(protected_account_router)
        .layer(auth_middleware.clone());

    let batch_upload_router = Router::new()
        .route("/api/photographs/batch-upload", post(batch_upload))
        .layer(DefaultBodyLimit::max(BATCH_REQUEST_SIZE));

    let authorization_admin_router = Router::new()
        .route(
            "/api/admin/authorization/users",
            get(list_authorization_users),
        )
        .route(
            "/api/admin/authorization/roles",
            get(list_authorization_roles),
        )
        .route(
            "/api/admin/authorization/permissions",
            get(list_authorization_permissions),
        )
        .route(
            "/api/admin/authorization/role-permissions",
            get(list_role_permissions),
        )
        .route(
            "/api/admin/authorization/audit",
            get(list_authorization_audit),
        )
        .route(
            "/api/admin/authorization/users/{user_id}/role",
            patch(assign_authorization_role),
        )
        .route(
            "/api/admin/authorization/roles/{role_id}/permissions/{permission_id}",
            patch(set_authorization_role_permission),
        )
        .layer(DefaultBodyLimit::max(AUTH_REQUEST_SIZE));

    let superuser_router = Router::new()
        .route("/api/admin/sync-i18n-cache", post(sync_i18n_cache))
        .route(
            "/api/admin/users/{user_id}/hard-purge",
            post(hard_purge_account),
        )
        .route("/api/admin/media-cleanup/unresolved", get(unresolved_media_cleanup))
        .route("/api/admin/media-cleanup/{cleanup_id}/resolve", post(resolve_media_cleanup))
        .route(
            "/api/admin/account-retention-notifications",
            get(retention_notification_status),
        )
        .route(
            "/api/admin/account-retention-notifications/{notification_id}/retry",
            post(retry_retention_notification),
        )
        .route("/api/blog/posts", post(submit_post))
        .route("/api/blog/{post_id}", patch(update_post))
        .route("/api/photographs/upload", post(upload_photograph))
        .route("/api/photographs/delete", delete(delete_photographs))
        .route("/api/photographs/batch/{batch_id}", get(batch_status))
        .route("/api/photographs/batches", get(batch_list))
        .route("/api/wasm-modules", post(upload_wasm_module))
        .route(
            "/api/wasm-modules/{wasm_module_id}",
            patch(update_wasm_module),
        )
        .route(
            "/api/wasm-modules/{wasm_module_id}/assets",
            post(update_wasm_module_assets),
        )
        .route(
            "/api/wasm-modules/{wasm_module_id}",
            delete(delete_wasm_module),
        )
        .merge(batch_upload_router)
        .merge(authorization_admin_router)
        .layer(require_superuser_middleware.clone())
        .layer(auth_middleware.clone());

    let api_router = public_router
        .merge(auth_abuse_router)
        .merge(oidc_callback_router)
        .merge(protected_oidc_router)
        .merge(protected_forum_router)
        .merge(protected_router)
        .merge(superuser_router)
        .layer(is_logged_in_middleware)
        .layer(trusted_origin_middleware)
        .layer(log_middleware)
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .layer(cors_layer)
        .with_state(state.clone());

    let router = Router::new().merge(api_router);
    let swagger_ui = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());

    let mut swagger_router = Router::new().merge(swagger_ui);

    if matches!(
        state.get_deployment_environment(),
        DeploymentEnvironment::Prod
    ) {
        swagger_router = swagger_router
            .layer(require_superuser_middleware)
            .layer(auth_middleware.clone());
    }

    let router = router
        .merge(swagger_router)
        .fallback_service(get(static_asset_handler));

    Ok(router.layer(compression_middleware))
}
