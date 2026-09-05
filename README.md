# cyhdev

This repository contains the backend, web application, maintenance utilities, and WebAssembly demos for cyhdev.

Clone with submodules, or initialize them after an existing checkout:

```bash
git clone --recurse-submodules https://github.com/younghyun1/cyhdev.git
git submodule update --init --recursive
```

## Packages

- `rust-be-template`: Axum and PostgreSQL backend.
- `solid-csr-spa-template`: SolidJS web application.
- `wasm_demos`: Browser demos built with Rust and WebAssembly.
- `vendor/eu5-location-filter`: Public Slint desktop/browser source pinned as a Git submodule.

## Development

Use `cargo xtask --help` for root-relative development, verification, and image commands. During implementation, `cargo xtask clippy` is the only stage gate; it checks native packages for the host and browser packages for `wasm32-unknown-unknown`. Run the broader final-review commands only after the implementation waves are complete.

Run `cargo xtask eu5-web-stage` to build the EU5 Locations DB development package with `wasm-pack --dev` and stage its host document, JavaScript, and WASM in the frontend's ignored public assets. Run the frontend development server afterward. Optimized EU5 WebAssembly builds are confined to the normal Docker pipeline.

## Optimized build

Run the restored compatibility entry point from any directory:

```bash
./build.sh
```

It is equivalent to `cargo xtask build`. The build uses locked frontend and Cargo inputs, digest-pinned nightly builders, `build-std`, the workspace release profile, gzip-9/zstd-22 frontend assets, and `target-cpu=znver3`. It exports the uncompressed glibc executable to `target/x86_64-unknown-linux-gnu/release/rust-be-template` and rejects unresolved dynamic libraries. `APP_NAME`, `TARGET_TRIPLE`, `TARGET_CPU`, `RUST_DOCKER_TAG`, `DOCKER_PLATFORM`, and `SOURCE_DATE_EPOCH` are the supported environment controls. Overriding `RUST_DOCKER_TAG` opts into a different host builder. The compatibility builder intentionally does not restore dependency updates or Git pulls.

`build.sh` and `cargo xtask image` validate the pinned EU5 submodule before starting Docker. Both local optimized paths build the EU5 browser package inside the container, stage it into the frontend, retain only gzip-9 representations under `/eu5-locations-db/app/`, and compile those assets into the backend executable. If the submodule is absent, initialize it with `git submodule update --init --recursive` and rerun the command.

Use `cargo xtask build-dev` when an unoptimized native development build is wanted. `cargo xtask image` separately builds the optimized musl deployment image.

The deferred gates are `cargo xtask fmt`, `cargo xtask unit`, `cargo xtask db-integration`, `cargo xtask migration-rollback`, `cargo xtask openapi`, `cargo xtask frontend-check`, `cargo xtask image-smoke`, `cargo xtask throughput`, and `cargo xtask secret-scan`. `cargo xtask final-review` runs all of them plus Clippy and reports every failed gate. It does not run a release build. Its throughput step first runs the portable fixture, then requires `THROUGHPUT_HTTP_TARGET`, `THROUGHPUT_HTTP_ENVIRONMENT`, and `THROUGHPUT_HTTP_THRESHOLDS` for the hardware-specific backend baseline; the environment must declare the exact target, and the thresholds must pin the observed environment digest. `THROUGHPUT_HTTP_OUTPUT` optionally changes the report destination.

Use `cargo xtask throughput record` to capture a threshold-free real-backend calibration before setting regression limits. Every `cargo xtask throughput` path compiles and executes the harness with the workspace release profile so harness debug overhead cannot distort the result. The [throughput runbook](tools/throughput/README.md) defines the localhost topology, calibration policy, and Wave 8 environment variables.

Database gates require `TEST_DATABASE_URL` to name a disposable PostgreSQL 18 maintenance database; the role needs `CREATEDB`, and the server must provide the trusted `uuid-ossp` extension. Tests create and remove uniquely named databases and never read `DATABASE_URL`. The frontend gate starts with `npm ci`. The image smoke gate pulls and builds the non-release Docker `smoke` target to validate the current build context, nightly toolchain, rust-src setup, and locked Cargo metadata; it does not compile or execute the optimized server. Secret scanning requires `gitleaks` and writes fully redacted current-tree and all-ref history reports under `target/secret-scan/`; every tracked or nonignored untracked candidate is snapshotted, unsupported or oversized candidates fail closed, and ignored runtime credentials are verified through Git and filesystem metadata without reading them.

`.gitleaks.toml` extends the upstream detector set. Its allowlists require both an exact fixture path and a deterministic dummy-value shape; production source and historical refs receive no rule-wide exceptions.

The checked-in `tools/final-review/evidence.manifest` is the executable W3/W8 evidence contract. The final review validates its xtask command and review-step registrations, required repository evidence, and the throughput and redacted secret-scan reports, then writes a deterministic receipt to `target/final-review/evidence.json`.

All root Cargo commands use `Cargo.lock`; frontend builds use `npm ci` and `package-lock.json`. Use `./build.sh` or the corresponding `cargo xtask` commands instead of package-local build scripts. Optimized backend and image commands verify digest-pinned nightly builders, build the frontend inside Docker, rebuild the Rust standard library, and apply the root release profile.

Frontend and backend build metadata use `SOURCE_DATE_EPOCH` when set. Root optimized and image commands otherwise use the current Git commit timestamp. The Docker orchestration transports that timestamp as `APP_BUILD_EPOCH`, limiting invalidation to steps whose output contains the metadata; direct Docker builds retain `SOURCE_DATE_EPOCH` as a fallback. Direct development builds without Git orchestration use their compilation time.

Runtime credentials stay in ignored environment or provider credential files. The repository does not provision external accounts, OAuth clients, or deployment secrets.
