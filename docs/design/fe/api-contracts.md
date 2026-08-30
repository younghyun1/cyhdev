# API contracts

The backend OpenAPI document is the source of truth for every HTTP request, response, path, query, and shared schema consumed by the frontend. The explicit browser surface is `FRONTEND_OPERATIONS` in `rust-be-template/src/openapi_codegen/surface.rs`; generation fails when a listed method and path is missing or mismatched, so adding a backend-only route does not silently expand the browser API. The current surface contains 59 operations in nine clients: account, reference data and health, blog posts, blog social actions, photography media, photography social actions, i18n, live-chat HTTP, and WebAssembly module management.

Run `npm run contracts:generate` in `solid-csr-spa-template` after changing a frontend-consumed `#[utoipa::path]` contract. It writes `src/generated/api-client.ts`, `src/generated/api-types.ts`, `src/generated/runtime.ts`, small clients under `src/generated/clients`, and one module per reachable schema under `src/generated/types`. Files and exports are ordered deterministically, unchanged files are not rewritten, and stale generated TypeScript is removed.

Run `npm run contracts:check`, or `cargo run --locked --package xtask -- openapi` from the repository root, to compare the complete generated tree in memory with the checked-in tree. Missing, changed, and stale files fail the drift check. Do not edit generated files directly.

`createApiClient` accepts a transport function instead of owning a base URL or credentials. The application adapter passes `apiFetch`, preserving cookie credentials, build headers, and unauthorized-session handling. Domain service wrappers preserve application-specific defaults, upload progress, and in-flight request sharing while deriving their HTTP types from this client. Multipart uploads use the generated request and response types through the progress-aware XMLHttpRequest helper; they do not define a parallel DTO.

JSON handlers using `http_resp` are documented as `{ success, data, meta }`. `/api/healthcheck/server` intentionally remains raw JSON, email verification remains text, and the WebAssembly bundle endpoint remains binary. Numeric `#[repr]` enums and fixed-length tuple responses are preserved as TypeScript literal unions and readonly tuples.

OpenAPI does not describe the WebSocket host-stat stream, live-chat WebSocket events, live-chat binary frames, WebRTC signaling, or the served WebAssembly bundle bytes. Those are binary or persistent protocol contracts rather than request-response HTTP DTOs; they remain explicitly typed in the frontend protocol modules.
