# cyhdev

This repository contains the backend, web application, maintenance utilities, and WebAssembly demos for cyhdev.

## Packages

- `rust-be-template`: Axum and PostgreSQL backend.
- `solid-csr-spa-template`: SolidJS web application.
- `wasm_demos`: Browser demos built with Rust and WebAssembly.

## Development

Use `cargo xtask --help` for root-relative development, verification, and image commands. During implementation, `cargo xtask clippy` is the only stage gate; it checks native packages for the host and browser packages for `wasm32-unknown-unknown`. Run the broader final-review commands only after the implementation waves are complete.

The deferred gates are `cargo xtask fmt`, `cargo xtask unit`, `cargo xtask db-integration`, `cargo xtask migration-rollback`, `cargo xtask openapi`, `cargo xtask frontend-check`, `cargo xtask image-smoke`, `cargo xtask throughput`, and `cargo xtask secret-scan`. `cargo xtask final-review` runs all of them plus Clippy and reports every failed gate. It does not run a release build. Its throughput step first runs the portable fixture, then requires `THROUGHPUT_HTTP_TARGET`, `THROUGHPUT_HTTP_ENVIRONMENT`, and `THROUGHPUT_HTTP_THRESHOLDS` for the hardware-specific backend baseline; the environment must declare the exact target, and the thresholds must pin the observed environment digest. `THROUGHPUT_HTTP_OUTPUT` optionally changes the report destination.

Use `cargo xtask throughput record` to capture a threshold-free real-backend calibration before setting regression limits. The [throughput runbook](tools/throughput/README.md) defines the localhost topology, calibration policy, and Wave 8 environment variables.

Database gates require `TEST_DATABASE_URL` to name a disposable PostgreSQL 18 maintenance database; the role needs `CREATEDB`, and the server must provide the trusted `uuid-ossp` extension. Tests create and remove uniquely named databases and never read `DATABASE_URL`. The frontend gate starts with `npm ci`. The image smoke gate pulls and builds the non-release Docker `smoke` target to validate the current build context, nightly toolchain, rust-src setup, and locked Cargo metadata; it does not compile or execute the optimized server. Secret scanning requires `gitleaks` and writes fully redacted current-tree and all-ref history reports under `target/secret-scan/`; every tracked or nonignored untracked candidate is snapshotted, unsupported or oversized candidates fail closed, and ignored runtime credentials are verified through Git and filesystem metadata without reading them.

`.gitleaks.toml` extends the upstream detector set. Its allowlists require both an exact fixture path and a deterministic dummy-value shape; production source and historical refs receive no rule-wide exceptions.

The checked-in `tools/final-review/evidence.manifest` is the executable W3/W8 evidence contract. The final review validates its xtask command and review-step registrations, required repository evidence, and the throughput and redacted secret-scan reports, then writes a deterministic receipt to `target/final-review/evidence.json`.

All root Cargo commands use `Cargo.lock`; frontend builds use `npm ci` and `package-lock.json`. Use `cargo xtask build`, `cargo xtask frontend-build`, `cargo xtask wasm-build`, and `cargo xtask image` instead of package-local build scripts. The image command pulls the rolling nightly builder, builds the frontend inside Docker, rebuilds the Rust standard library, and applies the root release profile.

Build metadata uses `SOURCE_DATE_EPOCH` when set and otherwise uses Unix epoch zero. Set the variable to a release timestamp when a meaningful build time is required without introducing wall-clock variation.

Runtime credentials stay in ignored environment or provider credential files. The repository does not provision external accounts, OAuth clients, or deployment secrets.
