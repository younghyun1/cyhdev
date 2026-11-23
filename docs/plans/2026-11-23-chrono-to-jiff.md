# Replace chrono with jiff 1.0

- Status: complete, 2026-11-23. Every direct use of `chrono` in the workspace now uses `jiff` 1.0, because `cargo audit` marks `chrono` unmaintained (RUSTSEC-2026-0543).
- Checkout: branch `chore/replace-chrono-with-jiff` from `e59a4a8c`.
- Completed: workspace and package manifests drop `chrono` and the Diesel and utoipa `chrono` features; `jiff` 1.0.2 with `serde` and `jiff-diesel` 1.0.0 with `postgres` replace them. utoipa's `jiff_0_2` feature matches field types by name and adds no jiff 0.2 crate, so it describes jiff 1.0 `Timestamp` fields. Diesel records use `deserialize_as`/`serialize_as` with the `jiff_diesel` wrappers; changesets hold the wrapper type directly because Diesel's `AsChangeset` derive does not honor `serialize_as`. Query binds go through `ToDiesel::to_diesel()`, and tuple loads read wrapper types and convert with `to_jiff()`. Durations are `SignedDuration`, with a day written as 24 hours, which is what chrono's UTC arithmetic did. UTC calendar helpers for the schedulers, EXIF parsing, and user-facing `+00:00` text live in `rust-be-template/src/util/time/utc.rs`.
- Verification: `cargo xtask clippy` (native and WASM), `cargo xtask fmt`, `cargo xtask unit`, and `cargo xtask openapi` exit 0, with no drift in `solid-csr-spa-template/src/generated/`. `cargo xtask db-integration` passed 51 cases, and `cargo xtask migration-rollback` passed 3 and 7, against a disposable PostgreSQL 18 container on loopback. `cargo tree --workspace -i chrono` shows only `oauth2` and `openidconnect`. Not run: `frontend-check` (no generated or frontend source changed), browser E2E, image smoke, and release builds.
- Remaining: none for this scope. `cargo audit` still reports RUSTSEC-2026-0543 until `openidconnect` and `oauth2` drop chrono upstream.

## Scope

`time` stays: the archive crates already use it next to jiff, and the backend passes `time::Duration` to axum-extra's cookie builder. `chrono` remains in `Cargo.lock` only as a transitive dependency of `oauth2` and `openidconnect`; no workspace crate names it.

## Behavior notes

JSON timestamps keep RFC 3339 with `Z`; jiff prints the fewest fractional digits needed where chrono padded to 3, 6, or 9 digits. Both forms parse identically. Strings that `to_rfc3339()` produced (emails, the archive reader list's grant time, the build timestamp) keep their `+00:00` offset through `rfc3339_utc`. Scheduler log fields print jiff's RFC 3339 form. EXIF years outside -9999..=9999 are now rejected as out of range; chrono accepted a wider range. Live chat binary frames and caches keep their millisecond and microsecond integers.
