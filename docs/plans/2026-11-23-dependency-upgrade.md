# November 23 dependency upgrade

Status: upgrade complete, November 23, 2026; the PGO and BOLT campaign result is recorded at the end.

Checkout: branch `chore/dependency-upgrade-2026-11-23`, based on `main` at `9ecf0c56`.

Completed: `cargo upgrade --incompatible` and `cargo update`. Every direct Rust dependency was already on its latest major, so the changes are patch and minor releases: anyhow 1.0.105, comrak 1.2.0, scc 3.9.3, the kcna-archive jiff requirement 1.0.2, and cookie 0.18.3, hashbrown 0.17.2, saa 5.6.5 and zip 9.8.2 in the lockfile. cookie 0.18.3 clears RUSTSEC-2026-0594 (private cookie nonces can repeat after a fork); the `private` feature was never enabled here. `num_cpus` is removed: its one caller, the Geo-IP bundle tool, now uses `std::thread::available_parallelism`. Pumpkin moved from `d0c0a5d` to [`a49be4b`](https://github.com/Pumpkin-MC/Pumpkin/commit/a49be4bee2113e4cf812de3117c61bc3e9d06d3f), still Java 26.3 data, and `GENERATOR_REVISION` names it; the eleven upstream commits touch advancement, enchantment and loot-table data, not biome generation. Frontend: dompurify 3.5.0, the `postcss-selector-parser` override 7.2.1, and `npm update` for 22 transitive packages.

Held back, unchanged from November 21: `@solidjs/testing-library` stays on 1.0.0-rc.2 because `latest` is the Solid 1 line; `@testing-library/jest-dom` stays on 7.0.1 because `@solidjs/vite-plugin` 3.0.1 ends its peer range at 7; Leaflet stays on 1.9.4 because 2.0 is beta; `generic-array` and `matchit` are pinned exactly by `crypto-common` and Axum.

Not replaced: `openidconnect` 4.1.0 is its latest release and still brings `rsa` 0.9 (RUSTSEC-2023-0071), `chrono` (RUSTSEC-2026-0543), `rand` 0.8 and `thiserror` 1. Removing it means hand-writing the OIDC code flow and ID token validation, which is a security-sensitive rewrite and was left for an explicit decision. `paste` (RUSTSEC-2024-0436) comes from `rav1e` through `image`. These three are the remaining `cargo audit` findings.

Verification: `cargo xtask clippy`, `cargo xtask fmt` and `cargo xtask openapi` pass. `cargo xtask unit` passes 678. `cargo test --locked -p minecraft-seed` passes 23, including the Paper climate fixtures. `cargo test --locked -p xtask` passes 86. `cargo xtask db-integration` (51) and `cargo xtask migration-rollback` (10) pass against a disposable PostgreSQL 18 container on loopback. `cargo xtask frontend-check` passes: typecheck, zero-warning lint, 529 and 17 Vitest tests, and the bundle. `npm audit` reports no vulnerabilities.

Not run: browser E2E, `image-smoke`.

## PGO and BOLT campaign

Pending: `./build_pgo_and_bolt.sh` from this branch head.
