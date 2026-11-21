# November 21 dependency upgrade

Status: complete, November 21, 2026. Upgraded every frontend and Rust workspace dependency to its latest resolvable release and promoted Solid 2 from release candidates to stable.

Checkout: `main` at `43340c0`, two commits ahead of `origin/main` before this plan; not pushed.

Completed: Solid moved from `solid-js`/`@solidjs/web` 2.0.0-rc.21, router 2.0.0-next.44, Vite plugin 3.0.0-next.56, and `@solid-primitives` next tags to the stable 2.0.3, 2.0.1, 3.0.1, keyed 3.0.0, and utils 7.0.1, still pinned exactly. No application source changes were needed. `eslint-plugin-solid` 0.20 maps JSX types to `@solidjs/web`, so the `solid/imports` rule is enabled again. Other frontend packages took their latest releases, including jsdom 31, Playwright 1.65, Vite 8.3.8, and Vitest 5.1.1.

Rust: `cargo upgrade --incompatible` and `cargo update`. Breaking-version bumps are diesel 2.4, diesel-async 0.10, diesel_migrations 2.4, axum-server 0.9, and comrak 1.1; the code compiled unchanged. comrak 1.x removed its `emojis` feature, which `shortcodes` now enables. Pumpkin moved from `5152bb5` to [`d0c0a5d`](https://github.com/Pumpkin-MC/Pumpkin/commit/d0c0a5d73d3757258f7afb2ca240279d52653862), still Java 26.3 data, and `GENERATOR_REVISION` now names that revision; it had still named the older `4426d11`. Its larger `MultiNoiseSampler` triggers `clippy::large_enum_variant` on the per-worker `Sampler`, which is suppressed with a reason rather than boxed. lettre 0.11.24 clears RUSTSEC-2026-0560 to 0565.

Held back: `@solidjs/testing-library` stays on 1.0.0-rc.2 because its `latest` tag is the Solid 1 line. `@testing-library/jest-dom` stays on 7.0.1 because `@solidjs/vite-plugin` 3.0.1 declares an optional peer range ending at 7, and npm refuses 8 without forcing peers. Leaflet stays on 1.9.4 because 2.0 is still beta. `generic-array` and `matchit` remain exactly pinned by `crypto-common` and Axum, as on October 30.

Correction: `postgres::i18n_sources` still asserted eight locale sources after the November 4 language expansion to 24. That suite had not been run since; the assertion and `rust-be-template/tests/README.md` now say 24.

Verification: `cargo xtask clippy`, `cargo xtask fmt`, `cargo xtask openapi`, and `cargo xtask unit` (517 passed) pass. `cargo test -p minecraft-seed` passes 23, including the independent Paper climate fixtures. `cargo xtask frontend-check` passes: typecheck, zero-warning lint, 422 Vitest tests in 57 files, and the bundle. `cargo xtask db-integration` (50 passed) and `cargo xtask migration-rollback` (10 passed) pass against a disposable PostgreSQL 18 container on loopback. Chromium E2E: 326 passed, 1 skipped, 1 failed.

The failure is `browser zoom viewport preserves original detail and coverage at 10800px` in `e2e/minecraft-prediction.spec.ts`, and it predates this upgrade. It fails 2 of 3 serial runs on rc.21 and 3 of 3 on 2.0.3. Mocked tiles become ready at about 20 per second under both versions, so the 1,056 tiles need about 50 seconds against a 45-second poll. Solid 2.0.3 also logs new `EFFECT_RELAY_TEAR` development warnings in the photograph viewer, where an effect copies `router.location` into `selectedPhoto` and `failedSrc`. These are warnings only. Not run: WebKit, `test:e2e:performance`, `test:e2e:layout`, `test:e2e:security`, `image-smoke`, and any release build.

Remaining: decide whether to raise the 10800px poll budget or speed up mocked tile delivery, and whether to turn the photograph viewer relays into memos. Remaining `cargo audit` findings have no upgrade available: `rsa` 0.9 through `openidconnect` 4.1, the latest release, plus `chrono` and `paste` marked unmaintained.
