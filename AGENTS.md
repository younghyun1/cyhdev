# cyhdev

Personal website workspace: Axum/PostgreSQL backend, SolidJS browser application, Rust WebAssembly demos, and Minecraft integration. The `*-template` directory names are historical; extend the existing application.

## Start here

1. Confirm checkout identity and remote state with `git status --short --branch`, `git remote -v`, `git fetch origin`, and `git log -5 --oneline`. Compare HEAD with its upstream; on `main`, compare with `origin/main`. If behind, fast-forward only a clean checkout. Preserve local work and report divergence or an unavailable remote instead of resetting. A feature branch need not equal `origin/main`.
2. Read the relevant row below and its scoped `AGENTS.md` before editing, including when starting from the repository root. Deeper instructions refine this file for their subtree; do not assume changing directories reloads them.
3. For resumed work, read the matching plan in `docs/plans/`, its last verification and next action, and the actual diff. Consult `TODO` for outstanding cross-project obligations. Plans and checkboxes describe intent; verify current code before treating work as pending or complete.
4. Load installed `rust-conventions` for Rust, `frontend-conventions` for browser work, and `database-conventions` plus its PostgreSQL reference for persistence work. Use native skill discovery or the installed `SKILL.md`; report missing required files. Project-specific choices below take precedence over generic stack defaults.

## Task map

For page discovery, navigation, and access boundaries, start with the [complete page flowcharts and route inventory](docs/design/fe/site-map.md). Update that map whenever routes, page guards, navigation groups, or embedded browser surfaces change.

| Task | Start with | Scoped instructions |
| --- | --- | --- |
| Backend behavior, HTTP, auth, jobs | `rust-be-template/src/features/`, `src/routers/main_router.rs`, `src/init/state/` | [Backend](rust-be-template/AGENTS.md) |
| Pages, state, CSS, browser protocols | `solid-csr-spa-template/src/routes.ts`, `src/app.tsx`, `src/services/` | [Frontend](solid-csr-spa-template/AGENTS.md) |
| Commands, CI, Docker, verification | `tools/xtask/src/main.rs`, `.github/workflows/clippy.yml`, `rust-be-template/Dockerfile` | [Tools](tools/AGENTS.md); also read it for root build changes |
| Browser Rust demos | `wasm_demos/block_breaker/`, `wasm_demos/ray_tracer/` | [WASM](wasm_demos/AGENTS.md) |
| Squaremap visibility plugin | `minecraft/map-control/README.md` | [Minecraft](minecraft/AGENTS.md) |
| EU5 browser app integration | `tools/xtask/src/eu5_web.rs`, `.gitmodules` | [Vendor](vendor/AGENTS.md) |
| Architecture, design, ongoing work | `docs/README.md`, `docs/plans/` | [Documentation](docs/AGENTS.md) |

Paths in the entry-point column are relative to the named package where abbreviated. Search the relevant subtree first. Avoid bulk-reading generated clients, migrations, vendored sources, or every design document.

## Commands and verification

Run these from the repository root. `Cargo.lock`, `rust-toolchain.toml`, `solid-csr-spa-template/package.json`, and its `package-lock.json` define actual versions. Rust uses rolling nightly; the frontend deliberately uses exact Solid 2 prerelease pins. Do not replace these with generic stable-stack defaults or upgrade dependencies as an incidental fix.

| Need | Command / prerequisite |
| --- | --- |
| Discover command behavior | `cargo xtask --help`; implementation is `tools/xtask/src/` |
| Browser setup and dev server | `npm --prefix solid-csr-spa-template ci`, then `cargo xtask frontend` (port 3000) |
| Backend dev server | `cargo xtask backend`; runtime files and configuration are backend-relative |
| Native development build | `cargo xtask build-dev` |
| Implementation stage gate | `cargo xtask clippy`; checks native and WASM targets separately |
| Final Rust checks | `cargo xtask fmt` (check only), `cargo xtask unit` |
| HTTP contract drift | `cargo xtask openapi` |
| Final frontend checks | `cargo xtask frontend-check`; installs locked dependencies, typechecks, lints, tests, and bundles |
| Database / rollback evidence | `cargo xtask db-integration`, `cargo xtask migration-rollback`; read backend instructions first |
| Docker / secret checks | `cargo xtask image-smoke` (Docker), `cargo xtask secret-scan` (`gitleaks`) |

Use Clippy as the implementation stage gate; run relevant deferred checks once the change is ready for final review. Browser E2E tests are separate from `frontend-check`. Documentation-only changes need path, command, scope, and `git diff --check` validation, not application builds. Report checks skipped and their reason; never describe an unrun gate as passing.

No release builds during this work. Do not run `build.sh`, `cargo xtask build`, `image`, `wasm-build`, `throughput`, or `final-review`: these invoke optimized profiles, including the throughput step hidden inside final-review. Run applicable non-release gates individually. The `:dev` image tag does not mean a development-profile build.

## Durable constraints

- Follow existing ownership and boundary documents. Handle errors explicitly, keep blocking work off Tokio workers, bound caches/queues/requests, and test changed behavior. Standalone automation belongs in workspace Rust binaries.
- HTTP contract changes must include backend annotations, regenerated TypeScript, and drift verification; see [API contracts](docs/design/fe/api-contracts.md). Never hand-edit `solid-csr-spa-template/src/generated/`.
- Keep credentials, certificates, runtime data, and generated build outputs out of Git. Do not read ignored credentials to perform routine onboarding. `TODO` records unresolved external credential rotation and dependency-promotion obligations; local checks do not close them.
- Use `sillok objective add`, `sillok note --parent <id>`, and `sillok objective complete` for substantive work. Record corrections as well as outcomes. Keep resumable project context in the relevant plan so it survives a machine change; follow [documentation instructions](docs/AGENTS.md).
- Commit coherent changes using existing `feat:`, `fix:`, `chore:`, `docs:`, or `test:` style. Preserve unrelated changes. Use `gh` for scoped PR work, verify author before writes, and follow the current user's external-communication restrictions. Repository instructions do not authorize deployment, live-service mutation, or sending messages.
- Write concise technical prose, one paragraph per line, without em dashes or self-attribution. Update instructions when their commands, ownership, or constraints change; keep volatile status in plans.

## Completion

Review the diff, record observed verification and remaining limitations, commit the scoped work, and update the relevant plan's next action or completed status. For interrupted work, leave an exact restart command, branch/commit, touched files, decisions, and blockers. Do not make the next session reconstruct these from conversation history.
