# Review correctness fixes

Status: complete for the scoped fixes, local verification, and publication on September 26. Fixed the confirmed request deadline, owner deletion, browser state, gallery pagination, and verification defects.

Checkout: implementation through `574bae0` was pushed to `origin/main` and verified by remote read-back. This completion record and its index links are the remaining documentation changes.

Completed: request deadlines apply through WebSocket handshakes; established sessions retain their separate limits (`cb2fd44`). Account deletion and role changes share ordered owner-row locks and reject removal of the final active owner with the existing HTTP 409 conflict (`4579691`). Photo social state is keyed by photograph, disposed callbacks cannot change later views, editable controls retain arrow keys, gallery page size stays fixed, and blog page bounds wait for matching server metadata (`8923822`). Evidence registrations are repaired and regression-tested; runtime reports require valid passing JSON, full review clears stale receipts, and a separate Validation workflow enforces non-release application checks (`574bae0`).

Verification: all checks below passed. Logs are `/tmp/cyhdev-fixes-*.log`. The isolated PostgreSQL 18.6 container was stopped after verification. No optimized build, deployment, or live-service mutation occurred.

Remaining: no scoped implementation defects remain. Inspect hosted Clippy and Validation outcomes with `gh run list --branch main`; their clean hosted execution is separate from the local results below. External credential rotation and artifact retirement remain open in `TODO`. WebKit, live RTC, new throughput measurements, and a fresh secret scan were not run.

## Verification results

| Command | Observed result |
| --- | --- |
| `cargo xtask clippy` | Native and WASM passed; existing manifest warnings for unused `bigdecimal` and `chrono-tz` remain. |
| `cargo xtask fmt` | Passed. |
| `cargo run --locked --package rust-be-template --bin openapi-contracts -- generate`, then `cargo xtask openapi` | Passed; no generated client drift. |
| `cargo xtask unit` | 363 passed; 60 ignored, including PostgreSQL and manual cases. |
| `TEST_DATABASE_URL=postgresql://postgres@127.0.0.1:55432/cyhdev_test_maintenance cargo xtask db-integration` | 47 passed across 23 suites on disposable PostgreSQL 18.6. |
| Same disposable database, `cargo xtask migration-rollback` | Nine passed. |
| `cargo xtask frontend-check` | Typecheck, zero-warning lint, 179 tests, and bundle passed; initial asset graph 117.8 KiB gzip. |
| `npm --prefix solid-csr-spa-template run test:e2e:chromium` | 217 passed; one live RTC case skipped. Eight new regressions are included. |
| `cargo xtask evidence` | Passed registration, source, and runtime report validation using existing runtime reports; this does not establish fresh throughput or secret-scan evidence. |
| Workflow and documentation checks | Actionlint, local Markdown links, and `git diff --check` passed. |

Corrections: the first resize regression assumed masonry DOM order; it now compares the full sorted photo inventory while still detecting skips and duplicates. Both resize cases passed after that correction, then the complete Chromium suite passed. The new hosted native job explicitly installs `rustfmt`, which the configured minimal toolchain does not include by default.

## Scope and decisions

- Apply request deadlines through the HTTP response, including WebSocket handshakes; upgraded sessions keep their separate session limits.
- Preserve at least one active owner across account deletion and role changes using the same database locking order.
- Bind optimistic photo state to its photograph, keep editing keys within editable controls, wait for pagination metadata before clamping blog routes, and keep gallery fetch page size stable.
- Repair the checked-in verification manifest and add regression coverage for its registrations. Add automated non-release validation beyond Clippy.
- Preserve the current language, framework, dependency, and retention choices. Historical credential rotation and artifact retirement remain external obligations in `TODO`; this work authorizes repository fixes and publication, not deployment or credential changes.

## CI references

Consulted September 26: [GitHub PostgreSQL service containers](https://docs.github.com/en/actions/tutorials/use-containerized-services/create-postgresql-service-containers), [official PostgreSQL image](https://github.com/docker-library/docs/blob/master/postgres/README.md), [Playwright CI](https://playwright.dev/docs/ci-intro), and [setup-node v7](https://github.com/actions/setup-node/releases/tag/v7.0.0). These informed service health checks, disposable credentials, browser dependency installation, and the pinned Node action. The exact maintenance database and step-scoped remote-CI override follow the repository's existing database guard.
