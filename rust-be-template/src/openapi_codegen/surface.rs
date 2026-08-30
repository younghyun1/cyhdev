/// One HTTP operation consumed by the browser application.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrontendOperation {
    pub client_group: &'static str,
    pub method: &'static str,
    pub path: &'static str,
}

macro_rules! operation {
    ($group:literal, $method:literal, $path:literal) => {
        FrontendOperation {
            client_group: $group,
            method: $method,
            path: $path,
        }
    };
}

/// Explicit frontend contract boundary. Missing or method-mismatched routes fail generation.
pub const FRONTEND_OPERATIONS: &[FrontendOperation] = &[
    operation!("account", "DELETE", "/api/auth/account"),
    operation!("account", "GET", "/api/auth/is-superuser"),
    operation!("account", "GET", "/api/auth/me"),
    operation!("account", "GET", "/api/auth/verify-user-email"),
    operation!("account", "GET", "/api/users/{user_name}"),
    operation!("account", "GET", "/api/admin/media-cleanup/unresolved"),
    operation!("account", "POST", "/api/auth/check-if-user-exists"),
    operation!("account", "POST", "/api/auth/login"),
    operation!("account", "POST", "/api/auth/logout"),
    operation!("account", "POST", "/api/auth/reset-password"),
    operation!("account", "POST", "/api/auth/reset-password-request"),
    operation!("account", "POST", "/api/auth/signup"),
    operation!("account", "PATCH", "/api/auth/profile"),
    operation!(
        "account",
        "POST",
        "/api/admin/media-cleanup/{cleanup_id}/resolve"
    ),
    operation!(
        "account",
        "POST",
        "/api/admin/users/{user_id}/hard-purge"
    ),
    operation!("account", "POST", "/api/user/upload-profile-picture"),
    operation!("account", "GET", "/api/user/profile-pictures"),
    operation!(
        "account",
        "POST",
        "/api/user/profile-pictures/{profile_picture_id}/select"
    ),
    operation!(
        "account",
        "DELETE",
        "/api/user/profile-pictures/{profile_picture_id}"
    ),
    operation!("reference", "GET", "/api/healthcheck/server"),
    operation!("reference", "GET", "/api/healthcheck/state"),
    operation!("reference", "GET", "/api/healthcheck/fastfetch"),
    operation!("reference", "GET", "/api/dropdown/language"),
    operation!("reference", "GET", "/api/dropdown/language/{language_id}"),
    operation!("reference", "GET", "/api/dropdown/country"),
    operation!("reference", "GET", "/api/dropdown/country/{country_id}"),
    operation!("reference", "GET", "/api/dropdown/country/{country_id}/subdivision"),
    operation!("reference", "GET", "/api/geolocate/{ip_address}"),
    operation!("reference", "GET", "/api/geo-ip-info/{ip_address}"),
    operation!("reference", "GET", "/api/geo-ip-info/me"),
    operation!("reference", "GET", "/api/visitor-board"),
    operation!("blog-posts", "GET", "/api/blog/posts"),
    operation!("blog-posts", "GET", "/api/blog/posts/{post_id}"),
    operation!("blog-posts", "GET", "/api/blog/search"),
    operation!("blog-posts", "POST", "/api/blog/posts"),
    operation!("blog-posts", "PATCH", "/api/blog/{post_id}"),
    operation!("blog-posts", "DELETE", "/api/blog/{post_id}"),
    operation!("blog-social", "POST", "/api/blog/{post_id}/vote"),
    operation!("blog-social", "DELETE", "/api/blog/{post_id}/vote"),
    operation!("blog-social", "POST", "/api/blog/{post_id}/{comment_id}/vote"),
    operation!("blog-social", "DELETE", "/api/blog/{post_id}/{comment_id}/vote"),
    operation!("blog-social", "POST", "/api/blog/{post_id}/comment"),
    operation!("blog-social", "PATCH", "/api/blog/{post_id}/{comment_id}"),
    operation!("blog-social", "DELETE", "/api/blog/{post_id}/{comment_id}"),
    operation!("photography-media", "GET", "/api/photographs/get"),
    operation!("photography-media", "POST", "/api/photographs/upload"),
    operation!("photography-media", "DELETE", "/api/photographs/delete"),
    operation!("photography-media", "POST", "/api/photographs/batch-upload"),
    operation!("photography-media", "GET", "/api/photographs/batch/{batch_id}"),
    operation!("photography-media", "GET", "/api/photographs/batches"),
    operation!("photography-media", "GET", "/api/photographs/{photograph_id}"),
    operation!("photography-social", "POST", "/api/photographs/{photograph_id}/vote"),
    operation!("photography-social", "DELETE", "/api/photographs/{photograph_id}/vote"),
    operation!("photography-social", "POST", "/api/photographs/{photograph_id}/{comment_id}/vote"),
    operation!("photography-social", "DELETE", "/api/photographs/{photograph_id}/{comment_id}/vote"),
    operation!("photography-social", "POST", "/api/photographs/{photograph_id}/comment"),
    operation!("photography-social", "PATCH", "/api/photographs/{photograph_id}/{comment_id}"),
    operation!("photography-social", "DELETE", "/api/photographs/{photograph_id}/{comment_id}"),
    operation!("i18n", "GET", "/api/i18n/ui-text"),
    operation!("i18n", "POST", "/api/admin/sync-i18n-cache"),
    operation!("live-chat", "GET", "/api/live-chat/messages"),
    operation!("live-chat", "GET", "/api/live-chat/cache-stats"),
    operation!("wasm", "GET", "/api/wasm-modules"),
    operation!("wasm", "POST", "/api/wasm-modules"),
    operation!("wasm", "PATCH", "/api/wasm-modules/{wasm_module_id}"),
    operation!("wasm", "POST", "/api/wasm-modules/{wasm_module_id}/assets"),
    operation!("wasm", "DELETE", "/api/wasm-modules/{wasm_module_id}"),
];
