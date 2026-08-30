# cyhdev

This repository contains the backend, web application, maintenance utilities, and WebAssembly demos for cyhdev.

## Packages

- `rust-be-template`: Axum and PostgreSQL backend.
- `solid-csr-spa-template`: SolidJS web application.
- `wasm_demos`: Browser demos built with Rust and WebAssembly.

## Development

Use `cargo xtask --help` for root-relative development, verification, and image commands. During implementation, `cargo xtask clippy` is the only stage gate; it checks native packages for the host and browser packages for `wasm32-unknown-unknown`. Run the broader final-review commands only after the implementation waves are complete.

All root Cargo commands use `Cargo.lock`; frontend builds use `npm ci` and `package-lock.json`. Use `cargo xtask build`, `cargo xtask frontend-build`, `cargo xtask wasm-build`, and `cargo xtask image` instead of package-local build scripts. The image command pulls the rolling nightly builder, builds the frontend inside Docker, rebuilds the Rust standard library, and applies the root release profile.

Build metadata uses `SOURCE_DATE_EPOCH` when set and otherwise uses Unix epoch zero. Set the variable to a release timestamp when a meaningful build time is required without introducing wall-clock variation.

Runtime credentials stay in ignored environment or provider credential files. The repository does not provision external accounts, OAuth clients, or deployment secrets.
