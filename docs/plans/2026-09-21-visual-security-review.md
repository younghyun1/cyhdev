# Visual review, chat moderation, and security checks

Status: complete for local implementation and fixture-based review on September 21, 2026; live-service visual verification remains an explicit limitation. Checkout: `main`, implementation commit `e449780`; the separate page map is `710ba8a`. Scope: visitor-map stacking, all SPA route renders, observed visual fixes, superuser live-chat deletion, and non-release performance/security review.

## Completed

- Visitor-map layers now have a local stacking context, attribution flows below the map, late requests cannot initialize a disposed map, and translated text is not inserted as HTML.
- Browser and desktop branding use `Young Hyun Chi | Software Engineer`, including both frontend and backend locale sources. Missing routes no longer appear under construction. Profile images preserve their aspect ratio beside long names; the public profile stacks cleanly on narrow screens.
- [Chat moderation](../architecture/be/live-chat-moderation.md) erases content using current database authority, invalidates the cache, and broadcasts deletion. Confirmation, failure feedback, protocol parsing, cache accounting, and persistence have focused tests. Generated HTTP contracts are synchronized.
- [Visual review and suggestions](../design/fe/2026-09-21-visual-review.md) describes all 132 fixture captures, fixes, remaining visual coverage, and proposed polish. [Performance/security findings](2026-09-21-performance-security-findings.md) records measurements and unresolved audit findings.

## Verification

- `cargo xtask clippy`, `cargo xtask fmt`, `cargo xtask unit`, and `cargo xtask openapi` passed. Clippy emitted existing unused-manifest-dependency warnings for `bigdecimal` and `chrono-tz`; ignored database/fuzz cases are not counted as executed by the unit gate. A redundant final unit rerun was terminated with status 143 and restarted separately.
- `cargo test --locked --package xtask`: 13 passed.
- `cargo xtask frontend-check`: typecheck, zero-warning lint, 104 unit tests, and budgeted Vite build passed. Final public-profile CSS changes were additionally typechecked/linted and recaptured.
- `TEST_DATABASE_URL=postgres://postgres@127.0.0.1:55439/cyhdev_test_maintenance cargo test --locked --package rust-be-template --test postgres_live_chat_moderation -- --ignored --nocapture`: one PostgreSQL 18 integration test passed. Only a disposable local container was used; it and its volume were removed afterward. Full database and rollback suites were not rerun; no schema migration was added.
- Chromium: the combined route-render, desktop, map, and moderation run passed 142 checks; subsequent avatar/icon changes passed another 136 render/map/moderation checks. Final profile layout passed a further 132-route recapture. The browser suite is fixture-based; WebKit and live backend/embedded-content checks were not run.
- `npm --prefix solid-csr-spa-template run test:e2e:performance`: six routes passed five-run mobile median thresholds. Full and production-only npm audits reported zero vulnerabilities. Rust advisories and the aggregate secret-scan limitation remain documented findings, not passing gates.
- `git diff --check` passed. No release builds or live-service mutations were performed; no open PR existed for this branch.

## Corrections and next action

The Rust locale-source test caught missing new keys in the backend JSON bundles after the frontend defaults and registry were updated. Both backend sources were corrected, including the old server-provided title; the unit gate then passed. The database harness rejected a bridged Docker address despite a loopback client URL; the disposable container was recreated with a loopback-only host listener without disabling the safety guard. Full-size inspection found a second, larger profile avatar also needed non-shrinking layout and a mobile stack.

To resume visual work, start with `npm --prefix solid-csr-spa-template run test:e2e:chromium -- page-renders.spec.ts --workers=4` and the visual review document. Generated PNGs and the local gallery are in ignored `target/page-renders/`; recreate them on another machine. Verify actual photographs, map tiles, EU5, Minecraft, Swagger, and WASM content with their respective running services before treating the live visual review as complete. Fix the secret scanner's Git-link handling and address dependency advisories in their owning upgrade paths. Existing external credential rotation in `TODO` remains mandatory before deployment.

French, Spanish, Simplified Chinese, Traditional Chinese, Japanese, and German translations remain separate future work. No translation rollout or live deployment occurred.
