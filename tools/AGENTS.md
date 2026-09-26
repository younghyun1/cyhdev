# Workspace tooling

Use workspace Rust binaries for standalone automation. `xtask/src/main.rs` owns command dispatch and native/WASM separation; `review.rs` owns deferred checks; `release.rs` owns optimized builds; `eu5_web.rs` owns EU5 staging. Read these files when changing root Cargo configuration, Docker builds, CI, or `build.sh` as well.

## Contracts

- Commands resolve paths from the workspace root and use locked dependency inputs. Preserve argument forwarding, exit status, contextual errors, and root-relative invocation.
- Register changed commands consistently in dispatch, help, tests, and `final-review/evidence.manifest` where applicable. `xtask/src/evidence_manifest.rs` validates that manifest; evidence receipts belong under ignored `target/`.
- `xtask/src/test_database.rs` guards disposable database selection. Preserve these guards and redaction in `xtask/src/secret_scan/`. Secret scans cover the working tree and all Git refs; do not broaden fixture allowlists to conceal findings.
- Root `.github/workflows/clippy.yml` runs the implementation gate only. `.github/workflows/validation.yml` separately runs formatting, native unit tests, HTTP contract drift, locked frontend checks, Chromium regressions, and PostgreSQL 18 integration/rollback against a disposable service. Neither workflow provides release, WebKit, Docker runtime, secret-scan, or hardware-specific throughput proof.
- `cargo xtask evidence` checks current registrations, source evidence, and passing JSON runtime reports; it does not establish the checkout identity of reports supplied separately. `final-review` removes prior runtime receipts before invoking their producers. The unit suite validates checked-in registrations and source paths without requiring ignored runtime reports.
- `throughput/README.md` owns benchmark topology, calibration, environment digest, and threshold policy. Never replace failed thresholds with a debug-profile benchmark or invent hardware-independent limits.

## Verification

Use `cargo test --locked --package xtask` for tooling changes and root `cargo xtask clippy` for the implementation gate. Formatting coverage in `cargo xtask fmt` includes the backend and xtask only; format/check other changed packages explicitly.

No release builds: `cargo xtask throughput` always passes `--release`, even for calibration. `final-review` calls it; `wasm-build` uses `wasm-release`; `image` and `build.sh` optimize the backend. Run relevant safe gates individually and report deferred evidence. `cargo xtask image-smoke` selects the non-release Docker `smoke` target but still requires Docker and network access. Do not edit benchmark or release behavior just to make documentation verification pass.

For frontend Docker input verification without Rust/EU5 release builds, run `docker build --file rust-be-template/Dockerfile --target frontend-source --tag cyhdev-frontend-source:check .`, then `docker run --rm --network none cyhdev-frontend-source:check npm run build`. This uses the same frontend sources and public translation catalogs as the deployment stage, but does not verify generated EU5 assets or final compression. Shared browser imports outside the frontend directory must have explicit narrow `COPY` instructions in that source stage; a successful host build does not prove container input completeness.
