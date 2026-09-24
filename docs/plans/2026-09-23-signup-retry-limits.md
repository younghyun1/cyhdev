# Signup retry limits

Status: complete, September 23. Increased signup retry allowances after production logs showed two invalid submissions exhausting the daily identity budget.

Checkout: `fix/performance-security-review` at implementation commit `de0d005`; policy, focused tests, and authentication documentation are committed. This completion record follows that commit.

Decision: raise each normalized email and user-name allowance from two to five attempts per day. Raise the source-IP allowance from three to ten per hour and from ten to twenty per day. Keep bounded tables, IPv6 /64 grouping, fixed-window expiry, and existing admission order.

Scope: policy limits only. Input validation still consumes signup attempts, and the browser still shows a generic failure message; those UX issues need separate changes. No deployment or live limiter reset.

Verification: `cargo test --locked --package rust-be-template --lib auth_signup_limits_tests` passed both tests. `cargo xtask clippy` passed native and WASM gates with the existing `bigdecimal` and `chrono-tz` unused-dependency warnings. `cargo xtask fmt`, documentation links, and `git diff --check` passed. `cargo xtask unit` passed 356 tests; logs are `/tmp/cyhdev-signup-unit.log`. Database and browser suites were not repeated because persistence and browser behavior are unchanged.

Next action: include this policy in the coordinated backend deployment. No scoped implementation remains; the running service still uses its deployed limits.

Reference: [authentication abuse boundaries](../architecture/be/authentication-abuse-boundaries.md).
