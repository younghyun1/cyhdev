# Minecraft map

Add a lazy `/minecraft` page to the Projects navigation. Embed squaremap at `/minecraft/map/` so its scripts and tiles load only while the page is open. Serve the plugin's public web directory directly through Axum using `SQUAREMAP_WEB_DIR`; no additional listener, proxy, or database is needed. Keep the route outside session and request-log middleware because map browsing generates many tile requests.

Use streaming filesystem responses with conditional requests. Revalidate assets and tiles because squaremap overwrites stable filenames; never cache live JSON responses. Return missing files as errors rather than the website shell. Restrict the configured root to squaremap's public `web` directory.

Verify routing, traversal rejection, conditional responses, live-data cache headers, frontend types, and lint. Installation must match the running Minecraft version. Production activation requires a normal website build and deployment; implementation checks use development builds only.

## Deployment

On `miniserver`, squaremap build [667](https://jenkins.jpenilla.xyz/job/squaremap/667/) (`1.4.1-SNAPSHOT+ac71dd2`, Minecraft 26.3) is installed in `/home/cyh/mcserver/plugins`. Its JAR SHA-256 is `0ffd4e13f171732a8fb296c0af715b2eac852e8309e152b9a16fdcc0452f0a5b`. Minecraft restarted successfully and logged that squaremap was enabled with its internal webserver disabled. The public web directory contains the interface and metadata for all three dimensions.

`/home/cyh/dist/.env` now sets `SQUAREMAP_WEB_DIR=/home/cyh/mcserver/plugins/squaremap/web`. Deploy the website through its normal optimized build workflow and restart its OpenRC service. No website production binary was replaced during implementation. Check `/minecraft`, `/minecraft/map/`, and `/minecraft/map/tiles/settings.json` after deployment. Serve only the generated public directory; do not place private files or symlinks to private directories there. The map is public and includes squaremap's default player markers.

The initial terrain render has not run. From an operator account in Minecraft, run `/squaremap fullrender minecraft:overworld`. Render the other dimensions afterward with `minecraft:the_nether` and `minecraft:the_end` if wanted. RCON is disabled and the supervised process has no interactive stdin, so SSH alone cannot issue these commands through the current service. Automatic background rendering is enabled for subsequent changes. Full renders use two threads; background rendering uses one thread and processes at most 128 chunks every 15 seconds. Player data updates every three seconds.

For rollback, remove the Projects link and unset `SQUAREMAP_WEB_DIR` to disable website access after restart. To remove squaremap, stop Minecraft, move its JAR out of `plugins`, and restart. Keep its data directory if the rendered map should be preserved. Do not expose the internal squaremap port; [squaremap supports serving generated files directly](https://github.com/jpenilla/squaremap/wiki/Internal-vs-External-Web-Server).

## Verification

The two Rust route tests pass: index and redirect behavior, conditional GET, HEAD, live JSON cache policy, missing files, traversal rejection, and an unconfigured deployment. Backend library Clippy passes with warnings denied. Frontend typecheck, ESLint, all 95 unit tests, and two Chromium tests covering desktop/mobile navigation, deferred map loading, and viewport bounds pass. The workspace Clippy gate fails in existing PostgreSQL integration tests, including `postgres_profile_picture_history.rs`, where `history.first()` resolves against Diesel's query extension trait. No production HTTP verification has run because the website binary has not been deployed.
