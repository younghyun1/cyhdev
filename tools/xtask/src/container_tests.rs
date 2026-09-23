//! Regression checks for the Docker build, runtime image, and local Compose files.

fn required_index(contents: &str, needle: &str) -> usize {
    let index = contents.find(needle).unwrap_or(contents.len());
    assert!(
        index < contents.len(),
        "required build instruction is missing: {needle}"
    );
    index
}

fn digest_pinned(image: &str) -> bool {
    image.split_once("@sha256:").is_some_and(|(name, digest)| {
        !name.is_empty()
            && digest.len() == 64
            && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

/// Every external base image must resolve to a content digest so a moved tag
/// cannot change what the build runs; `scratch` and earlier stages are local.
#[test]
fn container_base_images_are_pinned_by_digest() {
    let dockerfile = include_str!("../../../rust-be-template/Dockerfile");
    let mut global_arguments = Vec::new();
    let mut stages = Vec::new();
    for line in dockerfile.lines() {
        if let Some(argument) = line.strip_prefix("ARG ") {
            if stages.is_empty()
                && let Some((name, value)) = argument.split_once('=')
            {
                global_arguments.push((name, value));
            }
            continue;
        }
        let Some(from) = line.strip_prefix("FROM ") else {
            continue;
        };
        let mut words = from.split_whitespace();
        let image = words.next().unwrap_or_default();
        let stage = match (words.next(), words.next()) {
            (Some("AS"), Some(name)) => name,
            _ => "",
        };
        let expanded = global_arguments
            .iter()
            .fold(image.to_owned(), |expanded, (name, value)| {
                expanded.replace(&format!("${{{name}}}"), value)
            });
        assert!(
            expanded == "scratch"
                || stages.contains(&expanded.as_str())
                || digest_pinned(&expanded),
            "base image is not pinned by digest: {image}"
        );
        stages.push(stage);
    }
    assert!(stages.contains(&"smoke") && stages.contains(&"final"));
}

/// The scratch runtime has no shell and runs unprivileged, so every path the
/// backend writes must be created in a builder stage and copied with its owner.
#[test]
fn runtime_image_runs_unprivileged_with_owned_write_paths() {
    let dockerfile = include_str!("../../../rust-be-template/Dockerfile");
    let layout_start = required_index(dockerfile, " AS runtime-layout\n");
    let final_start = required_index(dockerfile, "FROM scratch AS final\n");
    assert!(layout_start < final_start);
    let layout = &dockerfile[layout_start..final_start];
    for instruction in [
        "mkdir -p /runtime/tmp /runtime/bin/logs /runtime/bin/data/search_index",
        "chmod 1777 /runtime/tmp",
        "chown 65532:65532 /runtime/bin/logs /runtime/bin/data/search_index",
    ] {
        assert!(
            layout.contains(instruction),
            "runtime layout is missing: {instruction}"
        );
    }

    let runtime = &dockerfile[final_start..];
    for instruction in [
        "WORKDIR /bin\n",
        "COPY --from=runtime-layout /runtime/ /\n",
        "COPY --chmod=0444 rust-be-template/new_bundle_ipv4.db rust-be-template/new_bundle_ipv6.db /bin/\n",
    ] {
        assert!(
            runtime.contains(instruction),
            "runtime image is missing: {instruction}"
        );
    }
    let user = required_index(runtime, "\nUSER 65532:65532\n");
    assert!(user < required_index(runtime, "\nCMD [\"/bin/server\"]"));
    assert_eq!(runtime.matches("\nUSER ").count(), 1);
}

#[test]
fn local_compose_is_loopback_only_and_confined() {
    let compose = include_str!("../../../rust-be-template/compose.yaml");
    let active = compose
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>();
    for required in [
        "      - 127.0.0.1:30737:443",
        "      - 127.0.0.1:3478-3541:3478-3541/udp",
        "    read_only: true",
        "      - ALL",
        "      - no-new-privileges:true",
        "      - /tmp:mode=1777,size=8g",
        "      - search-index:/bin/data/search_index",
        "      - logs:/bin/logs",
        "  search-index:",
        "  logs:",
    ] {
        assert!(active.contains(&required), "compose is missing: {required}");
    }
    let published = active
        .iter()
        .filter_map(|line| line.trim_start().strip_prefix("- "))
        .filter(|entry| entry.starts_with(|character: char| character.is_ascii_digit()))
        .collect::<Vec<_>>();
    assert_eq!(published.len(), 2);
    assert!(
        published
            .iter()
            .all(|entry| entry.starts_with("127.0.0.1:")),
        "every published port must bind loopback: {published:?}"
    );
}

#[test]
fn container_build_epochs_follow_stable_dependency_layers() {
    let dockerfile = include_str!("../../../rust-be-template/Dockerfile");
    let frontend_start = required_index(dockerfile, " AS frontend");
    let host_start = required_index(dockerfile, " AS host-release");
    let frontend = &dockerfile[frontend_start..host_start];
    assert!(
        required_index(frontend, "RUN --mount=type=cache,id=cyhdev-npm-cache")
            < required_index(frontend, "ARG APP_BUILD_EPOCH")
    );

    let artifact_start = required_index(dockerfile, "FROM scratch AS artifact");
    let host = &dockerfile[host_start..artifact_start];
    assert!(
        required_index(host, "rustup component add rust-src")
            < required_index(host, "ARG APP_BUILD_EPOCH")
    );
    assert!(
        required_index(host, "COPY --from=frontend") < required_index(host, "ARG APP_BUILD_EPOCH")
    );
}

#[test]
fn container_build_persists_expensive_package_caches() {
    let dockerfile = include_str!("../../../rust-be-template/Dockerfile");
    for cache_id in [
        "id=cyhdev-eu5-target",
        "id=cyhdev-eu5-cargo-registry",
        "id=cyhdev-eu5-cargo-git",
        "id=cyhdev-npm-cache",
    ] {
        assert!(
            dockerfile.contains(cache_id),
            "missing cache mount {cache_id}"
        );
    }
    assert!(
        include_str!("../../../.dockerignore")
            .lines()
            .any(|line| line == "**/logs")
    );
}

fn locked_wasm_bindgen_version(lockfile: &str) -> Option<&str> {
    let mut lines = lockfile.lines();
    lines.find(|line| *line == "name = \"wasm-bindgen\"")?;
    lines
        .next()?
        .strip_prefix("version = \"")?
        .strip_suffix('"')
}

/// The EU5 stage must never let wasm-pack fetch unpinned tool binaries.
#[test]
fn eu5_build_uses_integrity_checked_tools() -> Result<(), Box<dyn std::error::Error>> {
    let dockerfile = include_str!("../../../rust-be-template/Dockerfile");
    let stage_start = required_index(dockerfile, " AS eu5-wasm\n");
    let stage_end = required_index(dockerfile, " AS frontend-source\n");
    let stage = &dockerfile[stage_start..stage_end];
    for instruction in [
        "apk add --no-cache binaryen ",
        "cargo install wasm-bindgen-cli --version \"$WASM_BINDGEN_VERSION\" --locked",
        "wasm-opt --version",
        "wasm-pack build --release --mode no-install ",
    ] {
        assert!(
            stage.contains(instruction),
            "EU5 stage is missing: {instruction}"
        );
    }
    assert!(!stage.contains(".wasm-pack"));
    let pinned = stage
        .lines()
        .find_map(|line| line.strip_prefix("ARG WASM_BINDGEN_VERSION="))
        .ok_or("EU5 stage must pin WASM_BINDGEN_VERSION")?;

    // The Docker step enforces the same equality; this catches drift early
    // whenever the submodule is checked out locally.
    let lockfile = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/eu5-location-filter/Cargo.lock");
    match std::fs::read_to_string(lockfile) {
        Ok(contents) => assert_eq!(locked_wasm_bindgen_version(&contents), Some(pinned)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

#[test]
fn locked_wasm_bindgen_version_reads_the_package_entry() {
    let lockfile = "[[package]]\nname = \"wasm-bindgen-shared\"\nversion = \"9.9.9\"\n\n[[package]]\nname = \"wasm-bindgen\"\nversion = \"0.2.127\"\n";
    assert_eq!(locked_wasm_bindgen_version(lockfile), Some("0.2.127"));
    assert_eq!(locked_wasm_bindgen_version("name = \"serde\"\n"), None);
}

#[test]
fn container_context_excludes_local_outputs_and_tool_state() {
    let ignored = include_str!("../../../.dockerignore")
        .lines()
        .collect::<Vec<_>>();
    for pattern in [
        "solid-csr-spa-template/public/eu5-locations-db/",
        "**/test-results",
        "**/playwright-report",
        "rust-be-template/data",
        ".claude/",
        "**/CLAUDE.md",
    ] {
        assert!(
            ignored.contains(&pattern),
            "Docker context must exclude {pattern}"
        );
    }
}

#[test]
fn container_frontend_inherits_shared_locale_sources() {
    let dockerfile = include_str!("../../../rust-be-template/Dockerfile");
    let source_start = required_index(dockerfile, " AS frontend-source\n");
    let frontend_start = required_index(dockerfile, "FROM frontend-source AS frontend\n");
    let source = &dockerfile[source_start..frontend_start];
    let catalogs = required_index(
        source,
        "COPY rust-be-template/i18n/ui/ /workspace/rust-be-template/i18n/ui/",
    );
    assert!(required_index(source, "npm ci") < catalogs);
    assert!(source.contains("WORKDIR /workspace/solid-csr-spa-template"));
    assert!(source.contains("COPY solid-csr-spa-template/ ./"));
    assert!(!source.contains("--from=eu5-wasm"));
    assert!(!source.contains("COPY . ./"));
    assert!(!source.contains("COPY rust-be-template/ /workspace/rust-be-template/"));
    assert!(frontend_start < required_index(dockerfile, "RUN npm run build"));
}
