# cyhdev

This repository contains the backend, web application, maintenance utilities, and WebAssembly demos for cyhdev.

## Packages

- `rust-be-template`: Axum and PostgreSQL backend.
- `solid-csr-spa-template`: SolidJS web application.
- `wasm_demos`: Browser demos built with Rust and WebAssembly.

## Development

Use `cargo xtask --help` for root-relative development, verification, and image commands. During implementation, `cargo xtask clippy` is the only stage gate; it checks native packages for the host and browser packages for `wasm32-unknown-unknown`. Run the broader final-review commands only after the implementation waves are complete.

Runtime credentials stay in ignored environment or provider credential files. The repository does not provision external accounts, OAuth clients, or deployment secrets.
