# cyhdev PGO and BOLT campaign

`cargo xtask optimize` extends the existing GNU/Linux host build. It preserves the pinned Docker builder, embedded compressed frontend and EU5 assets, Zen 3 tuning, locked dependencies, rebuilt standard library, panic strategy, seed companion, and sampler licenses. The musl/UPX image and `build.sh` retain their existing behavior. This produces a candidate artifact; it does not install it or control a deployed service.

## Commands

Run these from the repository root. Inventory generation compiles the OpenAPI exporter in the development profile and checks its agreement with source route registrations. It does not contact a database, launch the application, or run fixture hooks.

```bash
cargo xtask optimize inventory target/optimization-inputs
cp target/optimization-inputs/campaign.template.json target/optimization-inputs/campaign.json
cp tools/optimization/config.example.json target/optimization-inputs/config.json
cargo xtask optimize plan target/optimization-inputs/config.json
```

The generated template currently contains 112 HTTP operations, 33 browser route states, and 31 scenario slots. The workspace [disposable fixture tool](../optimization-fixtures/README.md) prepares synthetic dependencies and generates a complete workload using the actual API contracts. Use that tool instead of the two template-copy commands when running the maintained campaign. The generic template remains deliberately incomplete for alternative fixtures: replace every placeholder, supply valid entity identifiers, order role-specific workflows, and write semantic success assertions. The default `/success` assertion does not apply to raw JSON, redirects, binary assets, or empty responses. Run `inventory` into a fresh directory after adding routes; it refuses to overwrite an existing template.

After reviewing the disposable environment and completing the campaign, commit all source changes and choose a fresh campaign name. The following command performs optimized builds and executes the supplied binaries. It is an operator command; ordinary development checks and CI do not invoke it.

```bash
cargo xtask optimize run target/optimization-inputs/config.json
```

`plan` prints the stages without building. `run` requires a clean commit and pinned clean EU5 submodule, rejects unfinished or incomplete campaign definitions before building, creates an exclusive run directory, and repeats identity checks throughout. Failed runs keep diagnostics and artifacts for inspection; use a new name to retry. Profiles are never imported from an earlier run. Changing source, configuration, or campaign during a run aborts acceptance.

## Runner and fixture environment

Use a dedicated native Linux x86-64 runner compatible with `target_cpu`; `znver3` remains the default deployment assumption. Docker Desktop emulation can build artifacts but cannot produce trustworthy native training or measurements, so `run` refuses other host architectures. The native runner must have the GNU libraries needed by the exported executable. The builder verifies its own dynamic dependencies; native startup separately verifies that the runner can execute it. Install locked frontend dependencies and the corresponding Chromium executable through the project's existing Playwright setup.

The runtime directory must be outside the checkout. Supply its public Geo-IP bundles, TLS certificate/key, disposable search index, synthetic media and WASM bundles, and backend environment variables. `IS_AWS_ECS=true` disables implicit `.env` loading. Secrets belong only in the dedicated process environment or private runtime files, never checked-in JSON. The browser driver forces loopback listeners, local mode, the supplied seed worker, and `PUBLIC_APP_ORIGIN` to the campaign origin. `HTTP_REDIRECT_PORT` makes the otherwise port-80 redirect listener independently configurable; its production default remains 80. The driver checks that both its HTTPS and redirect ports are free before resetting fixtures.

Required environment admission includes `CYHDEV_OPT_DISPOSABLE=1`, a loopback `DB_URL` selecting `cyhdev_optimization_*` without query overrides, a loopback SMTP sink, loopback `AWS_ENDPOINT_URL` and `OIDC_ISSUER_URL`, and image/SMTP credentials beginning with `optimization-fixture-`. PostgreSQL socket overrides are rejected. Use a local S3-compatible object store with the existing `cyhdev-img` bucket and a TLS/auth-capable SMTP fixture compatible with the application's relay transport. These gates prevent inherited production endpoints from being used accidentally; the fixture command remains trusted code and must enforce its own exact database/namespace checks.

Load the fixture member/admin passwords through `CYHDEV_OPT_MEMBER_PASSWORD` and `CYHDEV_OPT_ADMIN_PASSWORD`. Seed verified users with distinct ordinary/admin authority. The driver performs actual password logins and verifies database-current superuser state. Server sessions live in memory, so tokens from another server process are unsuitable. Browser census contexts share freshly established session state; each functional workflow creates a new login context. Account deletion, reset, and authority-changing workflows should use separate expendable accounts and recreate dependent sessions explicitly.

`reset_command` is an argv array for a workspace Rust fixture binary. It runs before every server start, including each benchmark stage, with the private runtime directory as its working directory. It must reset only the named disposable database, object store, search index, provider state, and synthetic integration data to the same recorded snapshot. It must never send mail to a human recipient or address a production socket. Scenario `command` steps also execute argv directly without a shell and must return success only after asserting their result. No fixture provisioning or production access is implied by these templates.

Minecraft socket paths, squaremap fixture files, and the search index must stay under the disposable runtime directory. Existing socket paths are checked after fixture reset to reject outward symlinks. Start synthetic same-user map-control/world-query adapters and a loopback management fixture at `minecraft_management_port`; its test secret is `optimizationfixture012345678901234567890`. Preserve the original visibility, permission, freshness, admission, and generator-profile behavior in these fixtures. Use synthetic seeds and worlds. Missing integrations are failures, not permission to relax feature gates.

## Coverage and workload design

The API inventory comes from the real OpenAPI document after the existing router drift check. The browser inventory is parsed from `src/routes.ts` with TypeScript's AST, including nested route normalization and inherited guards. A unit check compares it with [coverage.json](coverage.json), so adding a page requires a conscious coverage update. The live campaign visits each page on desktop/mobile, in light/dark themes, and in English/Korean; it checks the intended URL, a page-specific visible assertion, and JavaScript errors. Real backend responses and embedded build assets supply the pages; existing mocked browser suites cannot satisfy this gate.

Every API operation needs a successful explicit `request` step with matching method/path and a semantic JSON or binary assertion. `capture` maps fixture variable names to JSON pointers; later strings can use `${variable}`. Environment substitutions support private passwords and capability tokens without saving them in receipts. Multipart steps accept bounded fixture files. Browser steps support navigation, clicks, fills, uploads and keyboard actions followed by a visible/text assertion. WebSocket steps send bounded protocol messages and require expected replies. Negative 4xx cases may train error/authorization paths but never count toward successful operation coverage. A JSON error envelope also cannot count as success.

The 31 scenarios require the following fixture behavior; a name alone does not prove the assertions are sufficient:

| Area | Required behavior |
| --- | --- |
| Accounts | Signup and captured-mail verification; login/logout; password reset; profile/picture history; deletion and retention. Include failed credentials, expired capabilities, and revoked sessions using expendable identities. |
| OIDC and authorization | Local provider sign-in, link/unlink, callback state rejection, roles/grants/audit, denied member access, and refreshed/revoked session authority. |
| Blog and forum | Create/edit/publish/delete, search/pagination, nested comments/replies, votes/rescinds/shares, revisions/conflicts, moderation, subscriptions and notification reads. |
| Photographs and WASM | Multipart uploads, batch progress, thumbnail/metadata processing, filters/pagination/details, comments/votes, update/delete, and actual bundle serving/execution. Use representative fixture sizes and formats. |
| Reference, Geo and i18n | Country/subdivision/language lists, IPv4/IPv6 lookup, visitor reads, both UI locales, and administrative cache refresh. |
| Operations and lifecycle | Retention delivery to the mail sink, cleanup compensation/retry, hard purge, background processing, startup/cache warming, graceful shutdown and flushed profiles. |
| Host, chat and RTC | Health/state/statistics, host-stats WebSocket, chat history/send/moderation and session changes, plus call signaling and actual synthetic audio/video media exchange. Fake Chromium media devices are enabled. |
| Minecraft | All dimensions, tile formats/cache/visibility/profile expiry, hover, surveys, block reads/search, waypoint CRUD and permissions, and management actions against fixtures only. |
| Browser and assets | Navigation/auth guards and denied pages, compression/cache validators/malformed requests, EU5 sandbox behavior, and uploaded WASM execution. |

Define successful stateful workflows first, then add weighted repetitions and representative positive/negative variants. Broad coverage and realistic frequency serve different purposes: touching an administrative operation once should not make it hotter than ordinary page traffic. Avoid unconditional destructive steps in repeated workflows. The driver bounds workflows, actions, uploads, messages, request deadlines, and repetitions; each response is disposed immediately and responses above 16 MiB are rejected after Playwright materializes them. Because the HTTP client buffers responses, use controlled fixture responses; this is not an adversarial unbounded-response transport test.

## Artifact stages and acceptance

The pipeline builds a baseline, exercises full coverage, and benchmarks it; builds PGO instrumentation, exercises full coverage, and waits for clean shutdown; merges fresh profiles with the exact compiler's bundled `llvm-profdata`; rebuilds with profile-use; repeats coverage and benchmarking; instruments that exact PGO ELF with BOLT; runs the full campaign again; merges BOLT profiles; optimizes layout; strips the candidate; and repeats coverage and benchmarking on the stripped binary. It compares PGO with baseline and BOLT with both baseline and PGO before exporting anything under `dist/`.

All Rust variants use the same CPU, C flags, linker relocations, frame pointers, standard-library build, LTO and panic settings. Separate Cargo trees isolate the instrumentation phases. Compile-time PGO paths are absolute; `LLVM_PROFILE_FILE` points native executions at unique per-process raw profiles. The instrumented binary's compiled default also names the native run directory because the prediction supervisor clears its child's environment. `CARGO_ENCODED_RUSTFLAGS` preserves checkout paths containing spaces as single arguments. BOLT instrumentation writes to the absolute native run directory with PID suffixes and avoids requiring hardware branch sampling or elevated `perf` permissions. Invoke Debian's canonical `/usr/lib/llvm-19/bin/llvm-bolt` so its installation-relative runtime lookup finds `/usr/lib/llvm-19/lib/libbolt_rt_instr.a`; builder preparation checks and hashes that archive. BOLT receives an unstripped symbol table and text relocations; ELF checks require x86-64 PIE, resolvable libraries and final BOLT provenance. Any instrumentation/runtime/tool failure aborts the run without a silent fallback.

Raw profile inputs are limited to 4,096 regular nonempty files and 4 GiB per profile kind when collected; symlinks and unexpected PGO file types are rejected. PGO training also requires the prediction worker's `default_*.profraw` output; backend-only profiles cannot satisfy that gate. Put the run on a quota-controlled filesystem if it needs a hard live disk cap. Driver execution is deadline-bound; the built-in driver drains only its own backend with SIGTERM and rejects forced or unsuccessful exits. The outer driver process group is cleaned up on completion/failure/timeout. Neither path signals existing services. Reserve disk for three build trees, the builder/assets, profiles and retained binaries; optimized compilation itself uses Docker/Cargo's normal job resource controls.

The receipts retain commit, CPU, epoch, immutable builder image ID, compiler/tool versions and hashes, installed-package digest, exact BOLT input digest, raw profile hashes, per-stage binary hashes, complete coverage, and benchmark identities. Private server diagnostics remain under the run's `runtime-logs/`; traces/screenshots/video and response bodies are excluded from coverage receipts. Keep the run directory private.

Accepted output is `target/optimization/<name>/dist/rust-be-template`, its PGO seed companion, and `minecraft-seed-licenses/`. Baseline, PGO, and unstripped BOLT binaries remain alongside their evidence for comparison and rollback preparation. A failed campaign has no accepted receipt, even if partial output exists after an I/O failure. Installation, signing, publication and deployment remain separate operations.

## Measurements and routine checks

The built-in benchmark replays an explicitly weighted successful request list over reused HTTPS loopback connections, with warmup and 5-100 samples. It records the exact candidate, declared fixture/run conditions, observed CPU/kernel/memory/Node environment, workload, concurrency and topology. All stages must match environment/workload digests and sample counts, report zero failures, and pass median throughput/p95/p99 comparisons. Defaults allow at most 5 percent latency regression and 2 percent throughput regression; review budgets on the deployment hardware. Do not put credentials in `benchmark.environment`, which contributes to the recorded identity. Pin and verify the synthetic dataset/schema/object-store digest in the fixture reset binary; the generic browser driver does not independently dump a database to prove that declaration.

This benchmark includes Playwright/client work and its request transport; it is an acceptance comparison, not a calibrated capacity claim or proof of statistically significant improvement. The [existing throughput runbook](../throughput/README.md) remains available for its separately calibrated HTTP/1.1 public-read baseline. Collect independent production-shaped measurements after full functional acceptance before choosing an artifact to deploy.

```bash
cargo test --locked --package xtask
cargo xtask clippy
cargo xtask fmt
cargo xtask unit
cargo xtask openapi
npm --prefix solid-csr-spa-template run optimization:check
```

`cargo xtask frontend-check` and the existing Validation workflow include `optimization:check`. These tests use synthetic inputs and cover inventory, route matching, JSON pointers, isolation, missing coverage, identity mismatches, malformed reports, profile integrity, and regression decisions. They do not execute optimized binaries or certify fixture/provider/server runtime compatibility. The normal Playwright suite excludes this live campaign.

## Source consultation

Consulted September 29-30, 2026: [Rust PGO guidance](https://doc.rust-lang.org/rustc/profile-guided-optimization.html) establishes the instrumentation/merge/profile-use sequence, compiler tool selection and absolute paths; [Cargo's flag precedence](https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags) and [encoded environment variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html) establish argv encoding; [LLVM BOLT guidance](https://github.com/llvm/llvm-project/blob/main/bolt/README.md) establishes ELF symbols/relocations, instrumentation, profile merging and layout options; [Debian's BOLT 19 file list](https://packages.debian.org/trixie/amd64/bolt-19/filelist) and [LLVM 19 runtime lookup](https://github.com/llvm/llvm-project/blob/release/19.x/bolt/lib/RuntimeLibs/RuntimeLibrary.cpp) establish the packaged executable path and installation-relative runtime lookup. The runtime archive's actual location was verified inside the builder. The choice of the existing pinned GNU builder, `znver3`, and separate musl deployment behavior follows repository evidence. Debian package installations are recorded and the resulting image ID is reused within a run; the package repositories are not snapshot-pinned across separate image builds, so their receipts can differ even with the same Rust base digest.
