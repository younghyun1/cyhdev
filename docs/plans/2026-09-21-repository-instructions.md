# Repository instructions and session handoff

## Status

Completed 2026-09-21. Scope: provide a tracked root instruction file, scoped instructions at meaningful project boundaries, and a repeatable handoff convention. Implementation baseline: `main` at `25d4300`, confirmed equal to live `origin/main` before edits. Application behavior and dependency versions are unchanged.

## Research

Consulted current official documentation on 2026-09-21; this is a dated review of the available guidance, not a claim about a September-specific format release.

[Instruction discovery](https://learn.chatgpt.com/docs/agent-configuration/agents-md) documents root-to-working-directory loading, deeper precedence, one selected instruction file per directory, and a default combined 32 KiB limit. This informed the root routing table and compact scoped files. The root explicitly requires reading the destination subtree's instructions because a session launched at the root must not depend on automatic discovery of every descendant. No override files or local configuration changes are needed.

[Official best practices](https://learn.chatgpt.com/guides/best-practices) recommends concise, practical repository guidance covering layout, commands, conventions, constraints, and verification, with links for longer material. This informed the separation between durable instructions and changing plan state. The handoff fields are a repository-specific design to reduce repeated discovery; their effectiveness has not been benchmarked.

## Repository findings and decisions

- Removed the blanket `**/AGENTS.md` ignore so instructions survive a fresh checkout. Existing personal-tool and credential ignores remain.
- Added scoped files for backend, frontend, tooling, WASM demos, Minecraft, vendor integration, and documentation. No per-feature files are needed while those features share the same boundaries. No files were added inside the EU5 submodule.
- Linked existing architecture and design documents instead of duplicating them. Package READMEs and the Solid migration note contain historical examples or version pins; manifests, command implementations, and route registration are the authoritative lookup points.
- Documented the release-build trap: `final-review` invokes `throughput`, whose command explicitly uses `--release`; `wasm-build` uses `wasm-release`, and the deployment image optimizes despite its `:dev` tag. Existing root README wording that final-review does not run a release build is too broad. The new instructions require applicable non-release gates individually.
- Documented separate evidence boundaries: CI runs Clippy; frontend-check does not run Playwright; native unit tests exclude browser demos; root Cargo checks exclude the vendor repository and Java plugin.
- Preserved existing project choices, including mimalloc, Solid 2 prereleases, feature-local backend ownership, generated HTTP clients, and guarded disposable database tests.
- Added a handoff convention under `docs/AGENTS.md`: status, checkout, completed work, decisions, observed verification, blockers, and next action. The existing `TODO` remains the owner of standing credential-rotation and dependency-promotion obligations.

## Verification

`cargo xtask --help` passed. A read-only Node check validated 36 relative Markdown links across 11 documentation files, documented xtask commands against executable help (including aliases), and referenced npm scripts against `package.json`. `git check-ignore` confirmed that none of the eight instruction files is ignored. The largest root-plus-subdirectory chain is 10,132 bytes, below the default 32 KiB project budget; user-global instructions consume additional space. `git diff --check` passed. Application tests and builds were not run because this change only affects documentation and its Git tracking.

## Remaining

No implementation work remains after documentation validation. On the next substantive task, follow the root task map and maintain that task's own plan; do not reopen this completed plan as a work queue. External obligations in `TODO` remain unresolved. Measure onboarding quality through actual subsequent sessions before adding further rules.
