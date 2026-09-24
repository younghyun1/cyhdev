# Crate upgrade compatibility

Status: complete, September 23. Fixed compiler and Clippy failures caused by the dependency upgrade without downgrading any crate.

Checkout: `main` at implementation commit `53e785a`, following the two local AVIF-policy commits. The existing upgrades to `Cargo.lock`, `rust-be-template/Cargo.toml`, and `tools/throughput/Cargo.toml` were preserved byte for byte and committed with the compatibility fix. The initial patch is saved at `/tmp/cyhdev-user-crate-upgrade.patch` for comparison.

Completed: the initial Clippy gate found three compiler errors in `openapi_envelope.rs` because Utoipa 6 stores response media as `RefOr<Content>`. The modifier now unwraps inline media and preserves references; newly inserted JSON content is converted into the wrapper type. Regression tests cover envelope idempotence, null success data, and unchanged raw-health, binary, error, response-reference, and media-reference cases. No downgraded versions, suppressed diagnostics, or generated-client edits.

Verification: `cargo xtask clippy` passed native and WASM checks with only the existing `bigdecimal` and `chrono-tz` unused-dependency warnings. `cargo xtask fmt`, documentation links, and `git diff --check` passed. `CARGO_BUILD_JOBS=2 cargo xtask unit` passed 357 tests. `CARGO_BUILD_JOBS=2 cargo xtask openapi` passed without generated TypeScript drift. `RUSTFLAGS=--cfg=web_sys_unstable_apis CARGO_BUILD_JOBS=2 cargo test --locked --package block_breaker --package ray_tracer --lib` passed eight tests. Logs use `/tmp/cyhdev-upgrade-*.log`. PostgreSQL integration, live browser/media checks, and release builds were not run; the compatibility fix changes documented schema handling, with no persistence or wire-contract changes.

Next action: no scoped compatibility fixes remain. Publication and deployment remain separate; neither was performed.
