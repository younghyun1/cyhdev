# RTC runtime compatibility and reconnect verification

Status: complete, September 21, 2026. Checkout: `main`, integration baseline `45c0e8b`; initial compatibility fix `2865879` is pushed. Runtime fixes live in `rust-be-template/src/features/live_chat/service/rtc/` and `solid-csr-spa-template/e2e/rtc-live.spec.ts`.

## Findings and changes

Compilation did not detect RTC 0.21's new RTCP application-delivery boundary: PLI/FIR requests require `DeliverToApplication`. The last interceptor now forwards only these requests, leaving default transport/report handling intact. Both interceptor queues have explicit bounds and focused tests.

Existing subscriptions retained departed track IDs, preventing fresh publications from reaching browsers after rejoin. Teardown now cancels both publisher- and subscriber-owned forwarding/feedback tasks, removes senders and deduplication entries, and renegotiates remaining peers. A publication finishing initialization after teardown cancels itself. Join-scoped track IDs prevent overlapping reconnects from sharing deduplication entries; actor-based stream IDs still group the browser tiles. Failure teardown runs outside the driver callback so closing the driver cannot cancel room cleanup. Subscription maps have an explicit 128-track bound.

The real-browser test requires increasing inbound audio bytes and decoded video frames on both peers after ordinary leave/rejoin and WebSocket reconnect. It also checks mute/camera sender state, participant counts, and mounted video playback. An earlier nonzero-counter assertion was insufficient because it accepted historical media; it was replaced with fresh-counter comparisons.

## Local reproduction

Build the browser application with `npm --prefix solid-csr-spa-template run build` and the backend with `cargo build --locked --package rust-be-template --bin rust-be-template`. Run the backend against disposable PostgreSQL 18, temporary TLS, `RTC_ENABLE=true`, `RTC_PUBLIC_IP=127.0.0.1`, and an available UDP range. Do not load production environment files or credentials. The test deliberately rejects non-loopback URLs.

Provision and verify two accounts on that disposable backend: `rtcsmokea@example.test` and `rtcsmokeb@example.test`, both with the synthetic password `LocalSmokePass123`. Use normal registration and email verification against local test data; never provision these accounts on a real service. Then run `RTC_SMOKE_URL=https://127.0.0.1:3443 npm --prefix solid-csr-spa-template run test:e2e:chromium -- rtc-live.spec.ts --workers=1 --repeat-each=3 --output=../target/rtc-test-results`. Chromium fake media flags are configured by the test. Without `RTC_SMOKE_URL`, the test is skipped.

## Evidence and remaining work

Two feedback tests passed. Native/WASM Clippy and formatting passed, with the pre-existing unused `bigdecimal`/`chrono-tz` manifest warnings. The final implementation passed three consecutive real two-browser Chromium runs, including fresh audio/video after both forms of rejoin and two rendered video elements on each browser. Earlier development runs also passed; only the final repeats are completion evidence. Captures are under ignored `target/rtc-smoke/` and traces under `target/rtc-test-results/`.

No scoped implementation remains. Final integration and push evidence is in the [follow-up plan](2026-09-21-parallel-followups.md). This proves local synthetic audio/video transport and playback, not physical-device quality, remote NAT traversal, injected transport-failure cleanup, or production deployment. Driver-failure teardown and overlapping-join isolation were reviewed against the installed library's callback and cancellation behavior; the browser test exercises normal leave and WebSocket reconnect.
