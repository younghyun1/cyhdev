# Documentation and handoff

Keep backend architecture in `architecture/be/`, frontend design in `design/fe/`, and implementation plans in `plans/`. Follow these existing paths; this repository permits Markdown architecture documents. Link relevant documents from the existing indexes when adding them.

Instructions hold durable commands, ownership, and constraints. Plans hold changing scope, decisions, progress, evidence, blockers, and the next action. Avoid copying architecture prose or dependency versions into several instruction files. When docs disagree with executable code, inspect and correct the scoped discrepancy rather than following stale examples.

## Resumable work

For substantive multi-step work, create or reuse one relevant plan before implementation. Use `YYYY-MM-DD-descriptive-name.md` for new plans. Keep this compact handoff near its beginning and update it before stopping:

- Status: active, blocked, or complete; date and intended outcome.
- Checkout: branch, last relevant commit, and any uncommitted paths. Use the preceding implementation commit when recording the handoff itself.
- Completed: concrete behavior/files and decisions that constrain subsequent work.
- Verification: exact commands, observed results, prerequisites, and checks not run.
- Remaining: specific work or external blocker, then the next command/file to inspect.

Maintain the matching Sillok objective and notes, but do not make the plan depend on private log access. Record corrections to earlier assumptions explicitly. Use `TODO` for standing project obligations; do not infer completion from an old plan's checkbox or a passing local check. A completed plan must say so and must not appear as active in the plan index.

## Editing checks

Use concise prose with one paragraph per line and blank lines between paragraphs. No em dashes or self-attribution. Explain internal identifiers or link them in references. Keep credentials and private operational details out of documentation.

For documentation-only changes, verify relative links, referenced files, commands against their implementation, and `git diff --check`. No application build is needed. For researched guidance, record the consultation date, primary-source links, and which decisions follow from those sources versus local repository evidence. Keep research detail out of the root instruction loading path.
