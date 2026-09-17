# Minecraft map

Add a lazy `/minecraft` page to the Projects navigation. Embed squaremap at `/minecraft/map/` so its scripts and tiles load only while the page is open. Serve the plugin's public web directory directly through Axum using `SQUAREMAP_WEB_DIR`; no additional listener, proxy, or database is needed. Keep the route outside session and request-log middleware because map browsing generates many tile requests.

Use cached PNG tile responses with streaming filesystem fallback and conditional requests. Revalidate assets and tiles because squaremap overwrites stable filenames; never cache live JSON responses. Return missing files as errors rather than the website shell. Restrict the configured root to squaremap's public `web` directory.

## Tile memory cache

Cache PNG terrain tiles on demand in the existing `scc::HashCache`. Limit the combined allocation budget to 4 GiB: reserve 64 MiB for the bounded cache index and charge tile buffers plus conservative per-entry overhead against the remainder. Reservations stay attached to shared response buffers after eviction, so slow clients cannot cause unaccounted cached buffers. This is a cache allocation budget, not a process RSS limit; TLS, other application state, allocator overhead, and the operating system page cache remain separate.

Check file identity, size, and nanosecond modification/change timestamps on every lookup. Read misses through an open file and compare metadata before and after reading. Exclude JSON, oversized files, range requests, and unsupported paths from caching; preserve the existing file server as the fallback. Bound concurrent fills and cache entries independently, use bucket-local eviction from scc and bounded reclamation when the byte budget is exhausted, and never wait for cache space while serving a request. Verify eviction, in-flight buffer accounting, same-length replacement/deletion, concurrency, conditional requests, and fallback behavior with small test budgets. The [cache design](../architecture/be/minecraft-map-cache.md) records the implementation limits and HTTP behavior.

Verify routing, traversal rejection, conditional responses, live-data cache headers, frontend types, and lint. Installation must match the running Minecraft version. Production activation requires a normal website build and deployment; implementation checks use development builds only.

## Deployment

On `miniserver`, squaremap build [667](https://jenkins.jpenilla.xyz/job/squaremap/667/) (`1.4.1-SNAPSHOT+ac71dd2`, Minecraft 26.3) is installed in `/home/cyh/mcserver/plugins`. Its JAR SHA-256 is `0ffd4e13f171732a8fb296c0af715b2eac852e8309e152b9a16fdcc0452f0a5b`. Minecraft restarted successfully and logged that squaremap was enabled with its internal webserver disabled. The public web directory contains the interface and metadata for all three dimensions.

`/home/cyh/dist/.env` now sets `SQUAREMAP_WEB_DIR=/home/cyh/mcserver/plugins/squaremap/web`. Deploy the website through its normal optimized build workflow and restart its OpenRC service. No website production binary was replaced during implementation. Check `/minecraft`, `/minecraft/map/`, and `/minecraft/map/tiles/settings.json` after deployment. Serve only the generated public directory; do not place private files or symlinks to private directories there. The map is public and includes squaremap's default player markers.

The initial terrain render has not run. From an operator account in Minecraft, run `/squaremap fullrender minecraft:overworld`. Render the other dimensions afterward with `minecraft:the_nether` and `minecraft:the_end` if wanted. RCON is disabled and the supervised process has no interactive stdin, so SSH alone cannot issue these commands through the current service. Automatic background rendering is enabled for subsequent changes. Full renders use two threads; background rendering uses one thread and processes at most 128 chunks every 15 seconds. Player data updates every three seconds.

For rollback, remove the Projects link and unset `SQUAREMAP_WEB_DIR` to disable website access after restart. To remove squaremap, stop Minecraft, move its JAR out of `plugins`, and restart. Keep its data directory if the rendered map should be preserved. Do not expose the internal squaremap port; [squaremap supports serving generated files directly](https://github.com/jpenilla/squaremap/wiki/Internal-vs-External-Web-Server).

## Verification

All nine map cache and route tests pass: bounded allocation and eviction, response-buffer reservations, concurrent fills, file replacement/deletion, bypass conditions, index and redirect behavior, conditional GET, HEAD, range requests, live JSON updates, missing files, traversal rejection, and an unconfigured deployment. Backend library Clippy passes with warnings denied. The earlier frontend integration passed typecheck, ESLint, all 95 unit tests, and two Chromium tests covering desktop/mobile navigation, deferred map loading, and viewport bounds. The workspace Clippy gate previously failed in existing PostgreSQL integration tests, including `postgres_profile_picture_history.rs`, where `history.first()` resolves against Diesel's query extension trait. No production HTTP verification has run because the website binary has not been deployed.
