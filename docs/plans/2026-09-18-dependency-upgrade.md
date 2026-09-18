# Backend and frontend dependency update

The dependency sweep checked all first-party Cargo workspace packages and the frontend npm manifest against their registries. All 71 direct Rust dependencies are at their current stable release after upgrading `tokio-tungstenite` from 0.29.0 to 0.30.0. The Rust lockfile also advances `saa` from 5.6.0 to 5.6.1. Axum 0.8.9 still requires WebSocket transport 0.29 internally, so both WebSocket versions remain until Axum updates its dependency. This does not introduce another TLS implementation.

Frontend direct dependencies were already current. Solid 2 and its associated router, primitives, test library, and Vite plugin retain their exact compatible prerelease pins; several npm `latest` tags refer to older Solid 1 releases. The npm lockfile advances `@csstools/css-tokenizer` to 4.0.1, `baseline-browser-mapping` to 2.11.25, `electron-to-chromium` to 1.5.431, and `node-releases` to 2.0.56. The separately pinned EU5 submodule is outside the first-party workspace and was not modified.

## Transport and audit results

S3's default `rustls` feature still selects the legacy Hyper 0.14/Rustls 0.21 connector. Explicitly retain its other defaults (`default-https-client`, `http-1x`, `rt-tokio`, and `sigv4a`) and disable that legacy feature. AWS configuration now names `default-https-client` directly. The dependency tree contains only Hyper 1, h2 0.4, and Rustls 0.23 afterward. This removes the old h2 advisory and three old WebPKI advisories without changing credentials, endpoints, signing, or storage operations.

`npm audit` reports zero vulnerabilities. `cargo audit` retains [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html) through `openidconnect -> rsa 0.9.10`; the advisory has no patched release. The application's OIDC flow verifies provider signatures rather than exposing RSA private-key decryption. Tantivy 0.26.2 still requires `lru 0.16.4`, triggering [RUSTSEC-2026-0253](https://rustsec.org/advisories/RUSTSEC-2026-0253.html); the advisory requires a panicking key destructor, while Tantivy's store cache uses `usize` keys. Neither dependency finding is suppressed or represented as fixed. Audit also reports unmaintained indirect `bincode`, `paste`, and `yaml-rust` dependencies in the current image/Markdown stack. Removing these constraints requires upstream updates or a separately reviewed dependency replacement.

## Verification

The full native and WASM Clippy checks exposed test-only issues: a helper after a test module, Tungstenite's fixed large callback error type, and Diesel's `first` extension method shadowing slice access. The fixes move the helper, narrowly allow the upstream callback signature, and explicitly select slices in integration assertions. They do not change application queries or database behavior.

Frontend typecheck, full ESLint, all 102 unit tests, and all 32 Chromium browser tests pass. Native and WASM Clippy pass with warnings denied. The native workspace test run passes 190 tests with 34 explicit database/live-service/measurement cases ignored. Formatting and generated API contract drift checks pass. No release build or deployment was performed. Database and live-service tests retain their explicit opt-in requirements.
