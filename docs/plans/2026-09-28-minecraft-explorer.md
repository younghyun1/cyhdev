# Minecraft map exploration

- Status: complete, 2026-09-28, for local implementation and verification.
- Checkout: `main`, implementation through `e4a7fdb`; this completion record accompanies the documentation commit. Started clean and equal to fetched `origin/main` at `02fb3cb6b1250287f34f7a35e95e12bfeeb80858`. Local commits have not been pushed.
- Completed: bounded actual-world biome, elevation, block, and saved-structure queries; public named waypoints with administrator-only writes; native Solid/Leaflet terrain explorer with Minecraft-style controls, navigation, ruler, chunk grid, and portal coordinates. Original squaremap remains available in its existing sandbox. Fixed a mobile footer focus transition that could intercept form submission.
- Verification: Java compilation and fixtures, isolated Paper integration, Rust Clippy/format/unit/OpenAPI, frontend types/lint/unit/build, and disposable PostgreSQL integration/rollback pass. See the evidence table below for browser results and limitations. Both disposable servers were stopped cleanly.
- Remaining: no scoped implementation work remains. Production activation is a separate deployment step; start with the plugin README and activation notes below when authorized. No live plugin installation, database migration, service restart, or world mutation occurred. WebKit verification requires the missing host libraries noted below.

## Design

Keep squaremap as the terrain renderer. The Solid page uses Leaflet directly with squaremap's generated JSON and PNG data, without executing game-server-provided scripts in the authenticated website origin. Retain the original sandboxed map as an explicit fallback. Use scoped Minecraft-style beveled controls and inventory panels with native accessible form elements; preserve route-level lazy loading.

The Paper plugin exposes a separate same-user Unix world-query socket and preserves the existing visibility protocol. It reads generated chunks without generating new terrain, captures at most one snapshot per tick on the server thread, and inspects snapshots off that thread. Area queries cover at most 8 by 8 chunks with four-block surface samples; block searches cover at most 4 by 4 chunks and 512 vertical blocks. Responses report sampling time, scanned/missing chunk counts, and truncation. Queries run explicitly rather than continuously while panning. Biome height selection is separate from surface elevation.

The Rust API validates input and plugin output, bounds framing and deadlines, and serializes query work with a cooldown. Public waypoints live in the existing PostgreSQL database with strict constraints, bounded reads, and an enforced per-world count. Administrator writes use the established session, trusted-origin, and current-authority checks. All browser HTTP contracts are generated from OpenAPI.

Additional tools are coordinate navigation, a chunk grid, distance measurement, and Overworld/Nether coordinate conversion. Structure and block results describe the sampled generated area, not an unbounded whole-world index.

## Seed prediction

Seed predictions could fill unexplored areas behind actual terrain, with distinct visual treatment and provenance. They cannot represent player modifications, and generation version, dimension, datapacks, and world presets must match. No seed prediction is presented as actual world data or enabled without verified version support. This implementation does not require exposing or exporting the world seed.

Primary sources consulted on 2026-09-28: [squaremap API](https://github.com/jpenilla/squaremap#api), [squaremap client at the installed revision](https://github.com/jpenilla/squaremap/tree/ac71dd2/web/src/js), [Paper chunk snapshots](https://jd.papermc.io/paper/26.3/org/bukkit/ChunkSnapshot.html), [Paper generated structures](https://jd.papermc.io/paper/26.3/org/bukkit/Chunk.html#getStructures()), [Leaflet](https://leafletjs.com/reference.html), and [Cubiomes](https://github.com/Cubitect/cubiomes). The bounded actual-world scan design follows the local server integration and Paper API; the predicted-layer separation is a design recommendation.

## Verification and activation

| Command or check | Observed result |
| --- | --- |
| Plugin `javac --release 25 -Xlint:all,-classfile -Werror`, then `WorldProtocolTest` and `SavedStructuresTest` using the [README commands](../../minecraft/map-control/README.md#protocol-and-verification) | Passed against the actual Paper 26.3, squaremap, and Gson APIs; 53 protocol assertions and saved-region corruption, compression, numeric, and resource-limit fixtures passed. |
| `WorldDataBridgeIntegrationTest` against a fresh loopback-only Paper 26.3 build 28 instance | 579 checks passed, including real biome/elevation snapshots, block caps, saved structure metadata, both socket protocols, deadline/recovery, and permissions. The distant queried region file was absent before and after testing. Server stopped cleanly; both socket paths and listener disappeared. |
| `cargo xtask clippy` | Native and WASM passed. Existing unused `bigdecimal` and `chrono-tz` manifest warnings remain. |
| `cargo xtask fmt`, `cargo xtask openapi` | Passed; generated clients match backend contracts. |
| `cargo xtask unit` | 373 passed; 64 ignored external/manual cases. |
| `TEST_DATABASE_URL='postgresql://cyh@localhost/cyhdev_test_maintenance?host=%2Ftmp&port=55438' cargo xtask db-integration` | 50 passed across 24 suites on an isolated PostgreSQL 18.6 instance. Includes waypoint CRUD, concurrency/capacity, constraints, and administrator demotion races. |
| Same disposable database, `cargo xtask migration-rollback` | 10 passed, including refusing to drop retained waypoints and successful empty rollback/reapplication. The database instance was stopped after testing. |
| `cargo xtask frontend-check` | Typecheck, zero-warning lint, all 194 tests, and frontend bundle passed. |
| `npm --prefix solid-csr-spa-template run test:e2e:chromium -- --output=../target/minecraft-tests/chromium-results` | 220 passed; one live RTC case skipped. Includes public/admin waypoints, terrain/block queries, coordinate tools, navigation, and mobile layouts. |
| `npm --prefix solid-csr-spa-template run test:e2e:security -- --project=chromium --output=../target/minecraft-tests/security-results` | All 36 passed, including native explorer policy and original squaremap opaque-origin behavior. |
| WebKit security checks | Browser installation succeeded, but launch is blocked by unavailable host libraries `libicu74` and `libflite1`. No WebKit result is claimed. |
| Documentation and visual review | Desktop 1440px and mobile 390px screenshots inspected; changed documentation links resolve; `git diff --check` passed. |

Logs and browser artifacts are under ignored `target/minecraft-tests/`; fixture screenshots are `explorer-1440.png` and `explorer-390.png`. No release builds, live RTC checks, performance benchmark run, Docker image check, or fresh secret scan were required or run for this change.

Corrections: Paper's structure lookup can synchronously load referenced chunks, so saved own-start metadata is read with a bounded NBT parser instead. This excludes structures whose start chunk lies outside the survey and reflects the last save. Mobile browser testing exposed footer restoration between pointerdown and click; restoration now waits for gesture completion or cancellation. The first broad browser run shared Playwright's output directory with security tests, causing eight artifact cleanup failures; the final commands use distinct output directories. WebKit initially lacked its binary; installing it exposed the host-library prerequisite above.

Activation requires the updated plugin, `MINECRAFT_WORLD_SOCKET`, the embedded waypoint migration, and a normal website deployment. The isolated integration proves the query protocol against real test-world chunks; production data and deployment remain unverified. Waypoint display currently depends on the plugin catalog's dimension mapping, while base terrain and coordinate tools remain available without it. Existing credential-rotation obligations in `TODO` still apply to deployment. Seed prediction is a future separate layer; it is not implemented.
