# Minecraft map exploration

- Status: active, 2026-09-28. Add actual-world biome and elevation overlays, generated structures, bounded block search, public named waypoints with administrator-only editing, and map navigation tools.
- Checkout: `main`, starting at `02fb3cb6b1250287f34f7a35e95e12bfeeb80858`; clean and equal to `origin/main` after fetch.
- Completed: reviewed the existing sandboxed squaremap integration, visibility socket, public tile service, frontend conventions, and API contracts. Confirmed waypoint visibility with the user: public read, administrator-only create/update/delete.
- Verification: implementation checks pending; no production service or world changes authorized by this local implementation.
- Remaining: implement the Paper world-query socket, Rust query and waypoint APIs, and Solid explorer; run development checks and browser regressions; record activation prerequisites.

## Design

Keep squaremap as the terrain renderer. The Solid page uses Leaflet directly with squaremap's generated JSON and PNG data, without executing game-server-provided scripts in the authenticated website origin. Retain the original sandboxed map as an explicit fallback. Use scoped Minecraft-style beveled controls and inventory panels with native accessible form elements; preserve route-level lazy loading.

Extend the existing Paper plugin with a separate same-user Unix world-query socket. Preserve the existing visibility protocol unchanged. Read generated chunks without generating new terrain, capture a bounded number of snapshots on the server thread, and inspect snapshots off that thread. Limit area queries to 8 by 8 chunks with four-block surface samples and block searches to 4 by 4 chunks and 512 vertical blocks. Report sampling time, scanned/missing chunk counts, and truncated results. Queries run explicitly rather than continuously while panning. Biome height selection is separate from surface elevation.

The Rust API validates input and plugin output, bounds framing and deadlines, and serializes query work with a cooldown. Public waypoints live in the existing PostgreSQL database with strict constraints, bounded reads, and an enforced per-world count. Administrator writes use the established session, trusted-origin, and current-authority checks. All browser HTTP contracts are generated from OpenAPI.

Additional tools are coordinate navigation, a chunk grid, distance measurement, and Overworld/Nether coordinate conversion. Structure and block results describe the sampled generated area, not an unbounded whole-world index.

## Seed prediction

Seed predictions could fill unexplored areas behind actual terrain, with distinct visual treatment and provenance. They cannot represent player modifications, and generation version, dimension, datapacks, and world presets must match. No seed prediction is presented as actual world data or enabled without verified version support. This implementation does not require exposing or exporting the world seed.

Primary sources consulted on 2026-09-28: [squaremap API](https://github.com/jpenilla/squaremap#api), [squaremap client at the installed revision](https://github.com/jpenilla/squaremap/tree/ac71dd2/web/src/js), [Paper chunk snapshots](https://jd.papermc.io/paper/26.3/org/bukkit/ChunkSnapshot.html), [Paper generated structures](https://jd.papermc.io/paper/26.3/org/bukkit/Chunk.html#getStructures()), [Leaflet](https://leafletjs.com/reference.html), and [Cubiomes](https://github.com/Cubitect/cubiomes). The bounded actual-world scan design follows the local server integration and Paper API; the predicted-layer separation is a design recommendation.

## Verification and activation

Verify request limits and integer boundaries, absent chunks, response truncation, cancellation and stale browser replies, administrator authorization, waypoint persistence constraints, map projection, public marker rendering, narrow-screen layout, and independent original-map access. Run Java compilation against the installed APIs, backend Clippy/unit/contracts, frontend types/lint/unit/build, and mocked browser checks. Run disposable PostgreSQL tests only when the guarded test database prerequisite is available. No release builds.

Activation requires the updated plugin, the new world-socket environment setting, the waypoint migration, and a normal website deployment. Local compilation alone does not verify live chunk scanning or production world data. Existing credential-rotation obligations in `TODO` still apply to deployment.
