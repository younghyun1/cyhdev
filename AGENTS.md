# Agent Instructions

## Workflow
- In a git repo: commit regularly; match existing commit message style.
- When asked to do PRs using 'gh', title should be same as branch name, and description should be Changes and Rationale.
- Keep commentary terse, technical, token-efficient.
- Do not run release builds during agent workflows; dev checks/builds are fine.

## Sillok
Use Sillok for substantive agentic work. Record objectives, completed work, and corrections during the session.

- Assume `sillok` is on `PATH`.

Common commands:
```bash
sillok objective add "Finish storage/indexing refactor"
sillok note "Implemented archive-backed task logging" --parent <objective_id> --tags rust,agent
sillok note "Documented storage/indexing convention" --parent <objective_id> --tags docs
sillok objective complete <objective_id> --note "Scoped work is complete"
sillok tree --root <record_id>
```

## Rust
- No `unwrap()` or `expect()`.
- Handle `Option<T>` and `Result<T, E>` explicitly with `match`; use `if let` only when clippy/clarity warrants it.
- Generated code should avoid explicit panic paths. Log errors and gracefully exit the current context when exit is necessary.
- In `tracing` projects, use structured fields:
```rust
error!(env_key = key, error = %e, "Missing required environment variable");
```
- Performance matters: avoid Diesel/raw-SQL N+1s, unnecessary copies, and contended mutexes.
- Prefer `scc` or Tokio `RwLock` where contention is possible.
- Use `spawn_blocking` for blocking work inside Tokio.
- Linux/macOS only; `libc`/platform intrinsics are acceptable when faster.
- Run `cargo fmt` and `cargo clippy` after large refactors; fix warnings/errors.
- Split files above 300 LOC.
- Every folder should have `mod.rs` containing only module declarations.
- Check existing API/database conventions before adding APIs, response shapes, schemas, or models.
- Add rustdoc comments for modules/functions and useful inline commentary for non-obvious code.
- Drop objects with `drop()` immediately after final use when lifetime/resource release matters.

## Data Models
- Prefer UUIDv7 primary keys.
- Prefix column names with table names: `user_id`, not `id`.
- Unsigned autoincrement integers are acceptable only for very high row-count structures.
- Use Rust enums and rich domain types.
- Use `nutype` extensively for newtypes.
- Index all sortable columns and ID-like columns.
- Runtime-growing unbounded caches are forbidden unless the data is fixed reference data, such as ISO standards.

## Frontend
- Centralize styling in .css files so that all styles are defined in one place and consistent across the application.
- Use SolidJS router.
- Always use TypeScript strong typing, as strictly as possible.
- Keep dependencies up to date and use the latest stable versions.
- Use SolidJS context for global state management.
- Performance should be paramount.

## Tests
- Add unit tests wherever possible.
- Use `fuzztest` for advanced fuzzing.
- Use `gh` and other utilities as well as skills to review Copilot-generated code reviews on the PR if any are existent after each refactor pass. Fix, push, reply with fixed/rejected, cite details and hash in terse technical comments, close thread.

## Scripts
- Standalone project scripts should be Rust binaries in the same workspace.

## Docs
- Document backend design conventions in `/docs/architecture/be`.
- Document frontend design conventions in `/docs/design/fe`.
- Document plans in `/docs/planning`.
