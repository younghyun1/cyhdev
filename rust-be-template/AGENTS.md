# Backend

Read [feature boundaries](../docs/architecture/be/feature-boundaries.md) before structural changes. `src/features/accounts/` is the reference slice. Package README examples include historical routes and configuration; verify against `src/routers/main_router.rs`, `src/routers/main_router_registry.rs`, and `src/init/`.

## Ownership

- `features/<feature>/api`: Axum extraction, protocol validation, DTOs, cookies, status mapping; one service use case per handler. No Diesel or pool access.
- `service`: business rules, authorization, transaction intent, and post-commit cache/session coordination through explicit dependencies. No Axum, Diesel, or raw connections.
- `repository`: Diesel queries, pool checkout, transactions, persistence records, and domain conversion. No network work inside transactions or cross-feature repository imports.
- `domain`: persistence-independent values and rules. `src/persistence/` is limited to existing cross-feature relational primitives requiring a caller-owned transaction.
- `src/init/state/` composes dependencies; `src/jobs/` schedules maintenance. Keep business behavior off `ServerState`. Preserve the project's mimalloc choice rather than applying a generic allocator default.

Keep handwritten Rust files below 300 lines and `mod.rs` limited to declarations, following the documented generated-file exceptions. Use structured, redacted errors and logs. Bound runtime-growing storage; read [cache policy](../docs/architecture/be/runtime-cache-policy.md) when changing cache behavior.

## Contracts and persistence

For HTTP changes, read [API contracts](../docs/design/fe/api-contracts.md), update route registration and OpenAPI annotations together, and inspect `src/openapi_codegen/surface.rs` for browser exposure. From the root, regenerate with `cargo run --locked --package rust-be-template --bin openapi-contracts -- generate`, then run `cargo xtask openapi`. Preserve deliberate raw-JSON, binary, and WebSocket exceptions.

For persistence changes, load the database skill and PostgreSQL reference, then inspect `migrations/`, `src/schema.rs`, and the feature repository. Keep schema mappings and migrations aligned; use new migrations for deployed changes. Preserve rollback guards and retained-account/privacy invariants. Read the matching architecture document before changing account deletion, authorization, browser sessions, media persistence, or OIDC linking.

## Verification and runtime

Root `cargo xtask clippy` is the implementation gate. At final review, run `cargo xtask fmt`, `cargo xtask unit`, and contract checks as applicable. `tests/README.md` explains the ignored PostgreSQL suites: `TEST_DATABASE_URL` must select exactly `cyhdev_test_maintenance` on PostgreSQL 18 with `CREATEDB` and `uuid-ossp`. Tests create and drop isolated databases. Never substitute application `DB_URL` or bypass the remote-database guard. Run rollback coverage serially.

`cargo xtask backend` changes into this package before launching; `.env`, certificates, Geo-IP data, and index paths are backend-relative. Inspect configuration names without exposing values. Debug static assets use the frontend `dist/` path; if missing, inspect `src/routers/main_router/static_assets.rs` and prepare frontend assets through the root command. Missing runtime services are a reported prerequisite, not a reason to weaken checks.
