# Global request limits and fresh database latency

- Status: active, 2026-10-07; restore the historical HTTP request budget outside Minecraft and make each accepted health-state refresh measure PostgreSQL again.
- Checkout: `main`, starting at `e521f55`, synchronized with `origin/main`; no initial local changes.
- Completed: restored the historical 63 ms replenishment and 1,024-request burst with a 16,384-network table, atomic admission, debt-preserving expiry, and gated cleanup. One limiter covers HTTPS and HTTP redirects. Minecraft namespaces bypass that budget. Trusted proxy handling, normal JSON errors, retry hints, security headers, and allowed-origin error CORS are preserved. Health-state requests now perform fresh probes, with four active operations, a two-second caller deadline, cancellation-safe permits, and no HTTP storage. Runtime/cache and API contract documentation is updated.
- Verification: `cargo xtask clippy` passed native and WASM gates. Diff review and `git diff --check` passed. The first Clippy pass found one redundant test closure, corrected before the successful pass. Full unit tests, final formatting, and OpenAPI regeneration/drift checks remain in progress. No live services or release builds.
- Remaining: finish `cargo xtask unit`, `cargo xtask fmt`, `cargo run --locked --package rust-be-template --bin openapi-contracts -- generate`, and `cargo xtask openapi`; inspect any generated diff and commit the scoped work.

The limiter must cover API, documentation, static assets, fallbacks, and HTTP redirects, use the established trusted-proxy resolver, and aggregate IPv6 clients by /64. Preserve existing authentication and Minecraft admission rules. Keep the historical token budget without restoring unbounded Governor state. Fully replenished idle keys may expire; active keys must never be evicted to reset an abuse budget. At tracking capacity, reject novel keys rather than grow the table.

Minecraft namespaces `/api/minecraft`, `/api/admin/minecraft`, and `/minecraft`, including segment-delimited descendants, bypass only this shared request budget. Their existing authorization, body, connection, deadline, and worker limits still apply. Similar-looking prefixes must remain limited.

Database status must perform one fresh query per admitted health-state request, with bounded concurrency and caller waiting. A timed-out or cancelled HTTP request must not release probe admission while its database work still runs.
