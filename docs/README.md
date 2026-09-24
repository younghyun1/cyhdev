# cyhdev Documentation

This folder holds workspace-level implementation notes, plans, and engineering constraints for the `cyhdev` project family.

## Contents

- [Repository entry points and commands](../AGENTS.md)
- [Documentation and handoff conventions](AGENTS.md)
- [Complete page flowcharts and route inventory](design/fe/site-map.md)
- [Page render review and visual suggestions](design/fe/2026-09-21-visual-review.md)
- [Page layout and status metadata fixes](plans/2026-09-21-page-polish.md)
- [Performance and security findings](plans/2026-09-21-performance-security-findings.md)
- [September 23 performance and security fixes and verification](plans/2026-09-23-performance-security-review.md)
- [Signup retry allowances](plans/2026-09-23-signup-retry-limits.md)
- [AVIF encoding preference](plans/2026-09-23-avif-encoding.md)
- [September 23 crate upgrade compatibility](plans/2026-09-23-crate-upgrade-compatibility.md)
- [RTC, security, statistics, and localization verification](plans/2026-09-21-parallel-followups.md)
- [About page localization](plans/2026-09-22-about-localization.md)
- `architecture/be/` - backend architecture and boundary conventions.
- `design/fe/` - frontend interaction and visual design.
- `plans/` - implementation plans before code changes begin.

## Architecture

- [Backend feature boundaries](architecture/be/feature-boundaries.md)
- [Browser session security](architecture/be/browser-session-security.md)
- [Browser security headers and embedded applications](architecture/be/browser-security-headers.md)
- [HTTP and database runtime bounds](architecture/be/http-runtime-bounds.md)
- [Account identity and sessions](architecture/be/account-identity-and-sessions.md)
- [Authentication abuse boundaries](architecture/be/authentication-abuse-boundaries.md)
- [Account lifecycle and retained tombstones](architecture/be/account-lifecycle-retention.md)
- [Authorization administration](architecture/be/authorization-administration.md)
- [Authorization administration UI](design/fe/authorization-administration.md)
- [Media persistence](architecture/be/media-persistence.md)
- [OpenID Connect account linking](architecture/be/oidc-account-linking.md)
- [OpenID Connect account controls](design/fe/oidc-account-controls.md)
- [UI localization and translation coverage](design/fe/localization.md)
- [Forum architecture](architecture/be/forum.md)
- [WebAssembly service](architecture/be/wasm-service.md)
- [Live-chat moderation](architecture/be/live-chat-moderation.md)
- [Live-chat realtime limits](architecture/be/live-chat-realtime-limits.md)

## Active Plans

- [Live chat](plans/live-chat.md)
