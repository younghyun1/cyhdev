# Review correctness fixes

Status: active, September 26. Fix the confirmed request deadline, owner deletion, browser state, gallery pagination, and verification defects, then push the verified commits to `main`.

Checkout: `main` at `d50a1e5`, matching `origin/main` before implementation. The initial checkout is clean.

Completed: reviewed the history and current implementation. Fresh baseline verification passed 357 Rust tests, 179 frontend tests, frontend typechecking, and linting. Three targeted Chromium checks reproduced photo vote state leaking between photographs, comment arrow keys navigating photographs, and direct blog pagination resetting. Source inspection confirmed arbitrary `Upgrade` headers bypassing deadlines, self-deletion bypassing final-owner protection, and viewport changes altering pagination offsets. The evidence command fails on stale registration markers.

Verification: pending implementation. Run `cargo xtask clippy`, `cargo xtask fmt`, `cargo xtask unit`, `cargo xtask openapi`, `cargo xtask frontend-check`, focused Chromium regressions, and disposable PostgreSQL integration and rollback checks. Verify evidence registrations independently of missing performance and secret-scan receipts. No optimized builds or live-service changes.

Remaining: implement the fixes and regression tests; enforce applicable non-release checks in CI; review the combined diff; commit coherent changes; push `main` and inspect the resulting checks. Resume with `git status --short --branch` and this plan's verification record.

## Scope and decisions

- Apply request deadlines through the HTTP response, including WebSocket handshakes; upgraded sessions keep their separate session limits.
- Preserve at least one active owner across account deletion and role changes using the same database locking order.
- Bind optimistic photo state to its photograph, keep editing keys within editable controls, wait for pagination metadata before clamping blog routes, and keep gallery fetch page size stable.
- Repair the checked-in verification manifest and add regression coverage for its registrations. Add automated non-release validation beyond Clippy.
- Preserve the current language, framework, dependency, and retention choices. Historical credential rotation and artifact retirement remain external obligations in `TODO`; this work authorizes repository fixes and publication, not deployment or credential changes.
