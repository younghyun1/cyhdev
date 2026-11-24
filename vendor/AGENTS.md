# Vendored dependencies

`eu5-location-filter/` is a pinned Git submodule and is excluded from the root Cargo workspace. Root package checks do not verify its source. Keep host integration changes in `tools/xtask/src/eu5_web.rs`, the frontend EU5 page, and backend asset serving where possible.

Inspect `git submodule status` before use. If absent, initialize the pinned checkout with `git submodule update --init --recursive`. Do not use `--remote` or advance the gitlink as an incidental build fix. Preserve local submodule edits.

For development, root `cargo xtask eu5-web-stage` invokes `wasm-pack --dev` and replaces the ignored `solid-csr-spa-template/public/eu5-locations-db/app/` staging directory. Verify `wasm-pack` is installed. Do not hand-edit staged JavaScript or WASM; edit the source owner and rebuild.

When upstream source changes are explicitly in scope, read that repository's instructions and follow its host protocol document (`docs/design/fe/host-protocol.md`) and load Rust conventions for the iced user interface and Rust integration. Track its commit separately, then update the parent gitlink deliberately. Do not add parent-project instruction files inside the submodule or change its bundled datasets without a data-specific task.
