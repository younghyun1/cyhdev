# RTC, security, visual polish, and localization

Status: complete local implementation and verification, September 21, 2026. Checkout: `main`, implementation head `031f41e`; initial compatibility baseline `2865879` was pushed at start. Scope: local two-browser RTC smoke verification, confidentiality scanner and dependency-advisory fixes, statistics-page visual polish, and French/Spanish/Simplified Chinese/Traditional Chinese/Japanese/German UI translations.

Ownership is separated across RTC verification/integration, security tooling/dependencies, statistics components/styles, and localization files. Integration owns shared gate execution and commits. No release builds, production mutations, or confidential runtime files are required. Live-media/embedded-page visual review was explicitly dismissed by the user and is not a completion requirement; RTC media behavior testing remains requested.

## Delivered

- `f3c8a8a`: [recursive submodule confidentiality scanning and Markdown dependency pruning](2026-09-21-secret-scanner-dependencies.md). Removed two unused transitive dependency findings; retained three unsupported/unpatched upstream findings without suppressions.
- `5d981fa`: [statistics visual polish](2026-09-21-statistics-polish.md), semantic chart colors, consistent panels, and dark-mobile contrast correction.
- `45c0e8b`: [six additional UI translations](2026-09-21-localization.md), 650 keys per locale, fixed native-language selector names, lazy catalogs, fallback/race protection, cold-cache correction, and responsive authenticated-header layout.
- `031f41e`: [RTC runtime fixes](2026-09-21-rtc-runtime-verification.md), explicitly delivered keyframe feedback, bounded subscriptions, join-specific track IDs, and publisher/subscriber teardown cancellation.

## Verification

- `cargo xtask clippy` and `cargo xtask fmt`: passed; existing unused `bigdecimal` and `chrono-tz` manifest warnings remain. `cargo xtask unit` and `cargo xtask openapi`: passed; ignored database/fuzz cases are not counted as executed.
- `cargo xtask frontend-check`: typecheck, zero-warning lint, 115 tests, and bundle budget passed. Initial graph: 116.6 KiB gzip across seven assets, versus 124.3 KiB before this work.
- `npm --prefix solid-csr-spa-template run test:e2e:chromium -- page-renders.spec.ts localization.spec.ts statistics-polish.spec.ts --workers=4 --output=../target/final-browser-results`: 152 passed, including 132 route captures, all eight native-name selector states, offline/reload/switching behavior, and six statistics viewport/theme combinations. Images remain under ignored `target/page-renders/`, `target/localization-renders/`, and `target/statistics-polish/`.
- `RTC_SMOKE_URL=https://127.0.0.1:3443 npm --prefix solid-csr-spa-template run test:e2e:chromium -- rtc-live.spec.ts --workers=1 --repeat-each=3 --output=../target/rtc-test-results`: final implementation passed three consecutive runs. Both browsers receive fresh audio/video after ordinary leave/rejoin and signaling reconnect; video elements render synthetic streams.
- `npm --prefix solid-csr-spa-template run test:e2e:performance`: all six routes passed five-run cold-cache mobile median limits. LCP ranged from 1,112 to 1,412 ms; interaction measurements ranged from 32 to 64 ms.
- `TEST_DATABASE_URL=postgres://postgres@127.0.0.1:55440/cyhdev_test_maintenance cargo test --locked --package rust-be-template --test postgres_i18n_sources -- --ignored --nocapture`: passed, covering 5,200 translated rows, all eight cold/warm locale reads, and idempotent synchronization. No schema migration was added; the full database/rollback suite was not rerun.
- `cargo xtask secret-scan`: passed for current source, root history, and EU5 history, with zero findings in all three reports. The final commit must also pass before push. `git diff --check`: passed. No open main-branch PR existed for review.

## Limitations and next action

WebKit was unavailable locally. Physical-device audio quality, remote NAT traversal, and injected transport-failure cleanup are not proven by synthetic loopback RTC checks. Authored About essays/posts, embedded applications, and some browser-default date formatting remain unchanged; translations have no independent native-speaker review. Live-media/embedded-page visual verification remains dismissed, not silently counted as tested. Existing external credential rotation obligations in `TODO` remain distinct from local source checks. Three Rust dependency findings remain tracked in the security plan.

The disposable PostgreSQL container and volume, temporary synthetic TLS material, and local test server were removed/stopped. Only ignored screenshots and test evidence remain. No release build or deployment occurred. No scoped implementation remains; deployment readiness still requires the external obligations above. This handoff and the implementation commits are to be pushed after the post-commit confidentiality scan.
