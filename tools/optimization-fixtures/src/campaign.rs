//! Generate the complete checked workload against prepared synthetic fixtures.
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

pub fn write(runtime: &Path) -> anyhow::Result<()> {
    let output = std::env::current_dir()?.join("target/optimization-inputs");
    write_to(runtime, &output, "cyhdev-pgo-bolt-001", "znver3")
}

/// Managed builds use fresh inventory/config paths without overwriting previous campaigns.
pub fn write_to(runtime: &Path, output: &Path, name: &str, cpu: &str) -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    anyhow::ensure!(
        root.join("tools/optimization/coverage.json").is_file(),
        "run campaign generation from the workspace root"
    );
    for value in [name, cpu] {
        anyhow::ensure!(
            !value.is_empty()
                && value.len() <= 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')),
            "invalid campaign name or CPU"
        );
    }
    anyhow::ensure!(
        output.is_absolute()
            && output
                .canonicalize()?
                .starts_with(root.join("target/optimization-inputs")),
        "campaign output must be under target/optimization-inputs"
    );
    let fixture: Value = serde_json::from_slice(&fs::read(runtime.join("fixture.json"))?)?;
    let policy: Value =
        serde_json::from_slice(&fs::read(root.join("tools/optimization/coverage.json"))?)?;
    let surface: Value = serde_json::from_slice(&fs::read(output.join("surface.json"))?)?;
    let pages = policy["pages"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("missing pages"))?;
    let scenarios = policy["scenarios"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("missing scenarios"))?;
    let operations = surface["operations"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("missing operations"))?;
    let mut cases = BTreeMap::new();
    for page in pages {
        let pattern = page
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("invalid page"))?;
        let (selector, actor) = page_assertion(pattern);
        let mut case = json!({"selector":selector,"actor":actor});
        if pattern == "*404" {
            case["path"] = json!("/optimization-missing-page");
        }
        cases.insert(pattern.to_owned(), case);
    }
    let mut workflows = Vec::new();
    // API operation declarations are preflight inventory; successful calls are counted separately at runtime.
    for scenario in scenarios {
        let name = scenario
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("invalid scenario"))?;
        let declared: Vec<&Value> = operations
            .iter()
            .filter(|operation| operation.as_str().is_some_and(|op| owner(op) == name))
            .collect();
        workflows.push(json!({"name":name,"actor":actor(name),"steps":[{"kind":"fixture_scenario","name":name,"operations":declared}]}));
    }
    let login = |email: &str| {
        json!({"login":{"kind":"request","method":"POST","route":"/api/auth/login","path":"/api/auth/login","status":200,
        "body":{"user_email":email,"user_password":"OptimizationFixture123"},"json_pointer":"/success","equals":true}})
    };
    let benchmark_routes = [
        "/api/blog/posts",
        "/api/blog/posts/${post_id}",
        "/api/blog/search?q=fixture",
        "/api/forum/topics",
        "/api/photographs/get",
        "/api/dropdown/country",
        "/api/dropdown/language",
        "/api/i18n/ui-text?locale=en-US",
        "/api/healthcheck/state",
        "/api/visitor-board",
    ];
    let benchmark:Vec<Value>=benchmark_routes.into_iter().map(|path|json!({"kind":"request","method":"GET",
        "route":if path.contains("${post_id}"){"/api/blog/posts/{post_id}"}else{path.split('?').next().unwrap_or(path)},
        "path":path,"status":200,"json_pointer":"/success","equals":true})).collect();
    if let Some(flow) = workflows
        .iter_mut()
        .find(|flow| flow["name"] == "runtime.startup-shutdown-background-jobs")
    {
        let mut steps = Vec::with_capacity(1001);
        for _ in 0..100 {
            steps.extend(benchmark.iter().cloned());
        }
        steps.push(json!({"kind":"fixture_scenario","name":"runtime.startup-shutdown-background-jobs","operations":[]}));
        flow["steps"] = json!(steps);
    }
    let campaign = json!({"schema_version":1,"base_url":"https://127.0.0.1:18443","port":18443,"redirect_port":18444,
        "minecraft_management_port":crate::files::MANAGEMENT_PORT,"fixture_parameters":"fixture.json",
        "reset_command":[std::env::current_exe()?,"reset",runtime],"parameters":fixture["parameters"],
        "actors":{"anonymous":{},"member":login("fixture-member@example.test"),"admin":login("fixture-admin@example.test")},"pages":cases,"workflows":workflows,
        "benchmark":{"environment":{"fixture_snapshot_sha256":fixture["snapshot_sha256"],"fixture_inputs_sha256":fixture["inputs_sha256"],"run_conditions":"native Linux x86-64; disposable PostgreSQL and loopback providers; client-inclusive HTTPS"},
            "engine":"oha","worker_threads":4,"samples":9,"warmup_requests":10000,"requests_per_sample":100000,"concurrency":32,"actor":"anonymous","cases":benchmark}});
    fs::write(
        output.join("campaign.json"),
        serde_json::to_vec_pretty(&campaign)?,
    )?;
    let config = json!({"schema_version":1,"name":name,"target_cpu":cpu,"runtime_directory":runtime,
        "campaign":output.join("campaign.json"),"training_command":["npm","--prefix","solid-csr-spa-template","exec","--","playwright","test","--config","solid-csr-spa-template/playwright.optimization.config.ts"],
        "benchmark_command":["npm","--prefix","solid-csr-spa-template","exec","--","playwright","test","--config","solid-csr-spa-template/playwright.optimization.config.ts"],
        "timeout_seconds":7200,"maximum_latency_ratio":1.05,"minimum_throughput_ratio":0.98});
    fs::write(
        output.join("config.json"),
        serde_json::to_vec_pretty(&config)?,
    )?;
    crate::environment::write(runtime)?;
    Ok(())
}

fn actor(name: &str) -> &str {
    if name.starts_with("authorization.")
        || name.starts_with("operations.")
        || name.starts_with("i18n.")
        || name.starts_with("wasm.")
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

fn page_assertion(pattern: &str) -> (&str, &str) {
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
        "/forum/:topic_id" => (".forum-topic-detail__title", "anonymous"),
        "/photographs" => (".photo-card:not(.photo-skeleton-card)", "anonymous"),
        "/photographs/:photograph_id" => (".details-info", "anonymous"),
        "/visitor-board" => (".leaflet-container", "anonymous"),
        "/users/:userName" => ("main span.font-medium", "anonymous"),
        "/live-chat" => (".live-chat-page-heading", "anonymous"),
        "/minecraft" => (".leaflet-container", "anonymous"),
        "/eu5-locations-db" => ("iframe.eu5-locations-db-frame", "anonymous"),
        "/backend-stats" => (".backend-stats-page", "anonymous"),
        "/admin/minecraft" => ("main h1", "admin"),
        "/admin/authorization" => (".authorization-page h1", "admin"),
        "/admin/operations" => (".operations-page h1", "admin"),
        "/edit-profile" => ("main h1", "member"),
        _ => ("main h1", "anonymous"),
    }
}

fn owner(operation: &str) -> &str {
    let path = operation.split_once(' ').map_or("", |(_, path)| path);
    if path.starts_with("/api/admin/authorization") {
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
