# AVIF encoding preference

Status: complete, September 23. Every output variant uses the slowest AVIF preset with encoder-managed threading, prioritizing compression over upload processing time.

Checkout: `main` at implementation commit `96b8904`; the branch matched `origin/main` at `d9e3d47` before implementation. This completion record follows the implementation commit.

Decision: speed 1 for photographs, profile pictures, gallery thumbnails, and demo thumbnails; no per-encoder thread override. Quality remains 80. Pixel bounds and the two-job processing semaphore remain unchanged. Speed affects encoding effort and may change file size and decoded appearance; no measured size or visual-quality comparison is available.

Verification: `cargo test --locked --package rust-be-template --all-features --lib util::image::process_uploaded_image_files` passed three tests, including AVIF output for all four variants. `cargo xtask clippy` passed native and WASM gates with the existing `bigdecimal` and `chrono-tz` unused-dependency warnings. `cargo xtask fmt`, documentation links, and `git diff --check` passed. `CARGO_BUILD_JOBS=2 cargo xtask unit` passed 355 tests; logs are `/tmp/cyhdev-avif-unit.log`. The obsolete thread-cap test was removed. Database and browser suites were not repeated because persistence and browser behavior are unchanged. No release build or representative-photo benchmark was run.

Next action: include the policy in the next backend deployment. No scoped implementation remains. [Media persistence](../architecture/be/media-persistence.md) describes the implemented policy. Existing stored media will not be regenerated; no push or deployment was performed.
