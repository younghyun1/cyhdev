# Browser Rust demos

`block_breaker` and `ray_tracer` are workspace `cdylib` packages targeting `wasm32-unknown-unknown`. Each package has a browser host `index.html`; `ray_tracer/src/wgpu_renderer.rs` owns the GPU renderer. They are separate from the EU5 submodule and the backend's uploaded-WASM service.

Load Rust conventions for changes. Preserve browser event lifetimes, canvas sizing, animation cleanup, and graphics fallback behavior; test interaction in a browser when those change. Do not treat host-native compilation as proof of browser compatibility.

From the root, run `cargo xtask wasm-clippy` for focused verification; the full `cargo xtask clippy` also includes this check. It sets `RUSTFLAGS=--cfg=web_sys_unstable_apis` and the WASM target. `cargo xtask unit` excludes these packages; report browser validation separately. Check formatting with `cargo fmt --package block_breaker --package ray_tracer -- --check`.

Do not run `cargo xtask wasm-build` during development work; it uses the optimized `wasm-release` profile. When browser packaging is needed, verify `wasm-pack` is installed and use a package-specific `wasm-pack build --dev --target web` with the same WebGPU cfg and locked Cargo inputs. Keep generated `pkg/` output ignored.
