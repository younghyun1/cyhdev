//! Which scenario, actor, and page assertion the generated workload uses.
//!
//! Operations are assigned to scenarios by path, so a new route family needs an
//! arm here; an unassigned path would otherwise fall into the last scenario.

/// Page fixtures whose visited path is not the bare route pattern.
pub fn page_path(pattern: &str) -> Option<&'static str> {
    match pattern {
        "*404" => Some("/optimization-missing-page"),
        // Static About subpages have no seeded entity; one fixed page per
        // parameter route exercises the shared content renderer.
        "/about/:page" => Some("/about/experience"),
        "/about-blog/:page" => Some("/about-blog/performance"),
        // The comparison table only renders once terms are given; both occur
        // in the synthetic archive corpus.
        "/projects/kcna-archive/trends" => {
            Some("/projects/kcna-archive/trends?terms=lantern,harbor")
        }
        _ => None,
    }
}

pub fn actor(name: &str) -> &str {
    if name.starts_with("authorization.")
        || name.starts_with("operations.")
        || name.starts_with("i18n.")
        || name.starts_with("wasm.")
        || name.starts_with("kcna.")
        || name == "blog.crud-publishing-search"
        || name == "forum.moderation-subscriptions-notifications"
        || name == "minecraft.waypoint-crud"
        || name == "minecraft.server-controls-fixture"
        || name == "chat.history-send-moderation"
        || name == "browser.eu5-and-wasm-embeds"
        || name.starts_with("photographs.")
    {
        "admin"
    } else if name.starts_with("accounts.profile")
        || name.starts_with("oidc.")
        || name.starts_with("blog.")
        || name.starts_with("forum.")
        || name.starts_with("photographs.")
        || name == "rtc.signaling-call-media"
    {
        "member"
    } else {
        "anonymous"
    }
}

pub fn page_assertion(pattern: &str) -> (&str, &str) {
    match pattern {
        "/" => (".home-hero-title", "anonymous"),
        "/login" | "/register" | "/find-password" | "/reset-password" | "/verify-email" => {
            (".auth-card", "anonymous")
        }
        "/blog" => (".blog-list-title-row", "anonymous"),
        "/blog/new" | "/blog/:post_id/edit" => ("main h2", "admin"),
        "/blog/:post_id" => ("main h1", "anonymous"),
        "/forum" => (".forum-heading", "anonymous"),
        "/forum/new" | "/forum/notifications" => (".forum-heading", "member"),
        "/forum/:topic_id" => (".forum-topic-header__title", "anonymous"),
        "/photographs" => (".photo-card:not(.photo-skeleton-card)", "anonymous"),
        "/photographs/:photograph_id" => (".details-info", "anonymous"),
        "/visitor-board" => (".leaflet-container", "anonymous"),
        "/users/:userName" => (".user-info-name", "anonymous"),
        "/live-chat" => (".live-chat-page-heading", "anonymous"),
        "/minecraft" => (".leaflet-container", "anonymous"),
        "/eu5-locations-db" => ("iframe.eu5-locations-db-frame", "anonymous"),
        "/backend-stats" => (".backend-stats-page", "anonymous"),
        "/admin/minecraft" => ("main h1", "admin"),
        "/admin/authorization" => (".authorization-page h1", "admin"),
        "/admin/operations" => (".operations-page h1", "admin"),
        // The archive section is superuser-only; each selector matches one
        // element that exists only after the page's own data has loaded.
        "/projects/kcna-archive" => (".kcna-pane-head h2", "admin"),
        "/projects/kcna-archive/documents/:documentId" => (".kcna-document-title", "admin"),
        "/projects/kcna-archive/trends" => (".kcna-table", "admin"),
        "/projects/kcna-archive/status" => (".kcna-stats", "admin"),
        "/admin/kcna-archive" => (".kcna-access-list", "admin"),
        "/edit-profile" => ("main h1", "member"),
        _ => ("main h1", "anonymous"),
    }
}

pub fn owner(operation: &str) -> &str {
    let path = operation.split_once(' ').map_or("", |(_, path)| path);
    if path.starts_with("/api/kcna-archive") || path.starts_with("/api/admin/kcna-archive") {
        "kcna.search-read-trends-reload"
    } else if path.starts_with("/api/admin/authorization") {
        "authorization.roles-permissions-audit"
    } else if path.starts_with("/api/admin/account-retention")
        || path.starts_with("/api/admin/media-cleanup")
        || path.contains("/hard-purge")
    {
        "operations.retention-cleanup-purge"
    } else if path.starts_with("/api/admin/minecraft/map") || path == "/api/minecraft/map/waypoints"
    {
        "minecraft.waypoint-crud"
    } else if path.starts_with("/api/admin/minecraft") {
        "minecraft.server-controls-fixture"
    } else if path.starts_with("/api/admin/live-chat") || path.starts_with("/api/live-chat") {
        "chat.history-send-moderation"
    } else if path.contains("i18n") {
        "i18n.locales-refresh"
    } else if path.starts_with("/api/auth/oidc") {
        "oidc.login-link-unlink"
    } else if path == "/api/auth/signup" || path.contains("verify-user-email") {
        "accounts.signup-verification"
    } else if path.contains("reset-password") {
        "accounts.password-reset"
    } else if path == "/api/auth/account" {
        "accounts.deletion-retention"
    } else if path == "/api/auth/profile"
        || path.starts_with("/api/user")
        || path.starts_with("/api/users")
    {
        "accounts.profile-and-pictures"
    } else if path.starts_with("/api/auth") {
        "accounts.login-logout"
    } else if path.starts_with("/api/blog") {
        if path.contains("comment") || path.contains("vote") {
            "blog.comments-votes-shares"
        } else {
            "blog.crud-publishing-search"
        }
    } else if path.starts_with("/api/forum") {
        if path.contains("moderation")
            || path.contains("notification")
            || path.contains("subscription")
        {
            "forum.moderation-subscriptions-notifications"
        } else {
            "forum.topics-replies-revisions"
        }
    } else if path.starts_with("/api/photographs") {
        if path.contains("batch") || path.contains("upload") {
            "photographs.upload-batch-processing"
        } else if path.contains("comment") || path.contains("vote") || path.contains("delete") {
            "photographs.comments-votes-deletion"
        } else {
            "photographs.filters-details-metadata"
        }
    } else if path.starts_with("/api/wasm") {
        "wasm.upload-edit-serve-delete"
    } else if path.starts_with("/api/dropdown") {
        "reference.countries-subdivisions-languages"
    } else if path.contains("geo") || path == "/api/visitor-board" {
        "geo.lookup-and-visitor-board"
    } else if path.starts_with("/api/healthcheck") {
        "host.health-stats-websocket"
    } else if path == "/api/minecraft/map/query" {
        "minecraft.hover-surveys-blocks"
    } else {
        "minecraft.dimensions-tiles-visibility"
    }
}

#[cfg(test)]
#[path = "routing_tests.rs"]
mod tests;
