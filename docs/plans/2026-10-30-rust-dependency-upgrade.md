# Rust workspace dependency upgrade

Status: complete, October 30, 2026. Updated the first-party Rust workspace to current registry releases and refreshed its pinned Minecraft dependency.

Checkout: `main` at `8b06043`. The upgrade edits `Cargo.lock`, the root and package Cargo manifests, and this plan; commit the coherent set before stopping.

Completed: `cargo upgrade --incompatible` originally failed because Cargo was offline and the local index did not contain the selected wasm-bindgen macro release. Retried online with `CARGO_NET_OFFLINE=false cargo upgrade --incompatible --ignore-rust-version`, then updated the lockfile. The old Pumpkin revision pinned `sha2` exactly to 0.11.0, so the three Pumpkin crates now share upstream commit [`5152bb5`](https://github.com/Pumpkin-MC/Pumpkin/commit/5152bb5c1db05f63003fc7e1cd26ba293a45de41), which permits `sha2` 0.11.1. Cargo updated 71 registry and Git packages.

Verification: `cargo xtask clippy`, `cargo xtask fmt`, and `CARGO_BUILD_JOBS=2 cargo xtask unit` passed. Clippy retains two existing unused-dependency warnings for `bigdecimal` and `chrono-tz`. `cargo update --dry-run --verbose --ignore-rust-version` reports only two newer registry versions that cannot resolve: `generic-array` is exactly pinned by `crypto-common` 0.1.7, and `matchit` is exactly pinned by Axum 0.8.9. Attempts to force either version confirmed the upstream constraints.

Remaining: no scoped implementation work. Database-backed and live-service tests remain skipped by their existing prerequisites; no release build was run.
