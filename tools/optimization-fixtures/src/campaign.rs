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
        let (selector, actor) = crate::routing::page_assertion(pattern);
        let mut case = json!({"selector":selector,"actor":actor});
        if let Some(path) = crate::routing::page_path(pattern) {
            case["path"] = json!(path);
        }
        cases.insert(pattern.to_owned(), case);
    }
    let mut workflows = workflow_cases(scenarios, operations)?;
    let login = |email: &str| {
        json!({"login":{"kind":"request","method":"POST","route":"/api/auth/login","path":"/api/auth/login","status":200,
        "body":{"user_email":email,"user_password":"OptimizationFixture123"},"json_pointer":"/success","equals":true}})
    };
    let benchmark = benchmark_cases();
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

/// API declarations are preflight inventory; successful calls are counted at runtime.
fn workflow_cases(scenarios: &[Value], operations: &[Value]) -> anyhow::Result<Vec<Value>> {
    let mut workflows = Vec::with_capacity(scenarios.len());
    for scenario in scenarios {
        let name = match scenario.as_str() {
            Some(name) => name,
            None => anyhow::bail!("invalid scenario"),
        };
        let declared: Vec<&Value> = operations
            .iter()
            .filter(|operation| {
                operation
                    .as_str()
                    .is_some_and(|op| crate::routing::owner(op) == name)
            })
            .collect();
        workflows.push(json!({"name":name,"actor":crate::routing::actor(name),"steps":[{"kind":"fixture_scenario","name":name,"operations":declared}]}));
    }
    Ok(workflows)
}

/// Capacity traffic excludes the four-slot fresh database probe; its scenario still checks it.
fn benchmark_cases() -> Vec<Value> {
    [
        "/api/blog/posts",
        "/api/blog/posts/${post_id}",
        "/api/blog/search?q=fixture",
        "/api/forum/topics",
        "/api/photographs/get",
        "/api/dropdown/country",
        "/api/dropdown/language",
        "/api/i18n/ui-text?locale=en-US",
        "/api/healthcheck/server",
        "/api/visitor-board",
    ]
    .into_iter()
    .map(|path| {
        let route = if path.contains("${post_id}") {
            "/api/blog/posts/{post_id}"
        } else {
            match path.split_once('?') {
                Some((route, _)) => route,
                None => path,
            }
        };
        let mut request =
            json!({"kind":"request","method":"GET","route":route,"path":path,"status":200});
        if route == "/api/healthcheck/server" {
            // Server health returns raw build metadata without a success envelope.
            request["minimum_bytes"] = json!(16);
            request["content_type"] = json!("application/json");
        } else {
            request["json_pointer"] = json!("/success");
            request["equals"] = json!(true);
        }
        request
    })
    .collect()
}

#[cfg(test)]
#[path = "campaign_tests.rs"]
mod tests;
