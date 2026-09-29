# Minecraft browser-zoom tile coverage

Status: complete, 2026-09-29. Missing prediction tiles at extreme browser zoom are repaired locally and mirrored to the public integration repository.

Checkout: website `main` at implementation commit `11189aa` and overview regression commit `8806665`, seven commits ahead of fetched `origin/main` before this documentation commit. Public integrations are pushed at [`037858a`](https://github.com/younghyun1/cyh_minecraft_plugins/commit/037858ab15fdff25ce4e6f3d6b655b20ee5ea474). The Minecraft service was not touched.

Diagnosis: prediction and highlight layers always used 256 CSS-pixel tiles. A 7,200 by 4,050 CSS-pixel viewport requires roughly 600 tiles with Leaflet's buffer, while the tile store admits at most 256 active targets; rejected subscriptions leave blank canvases. The image's isolated prediction strips match this failure mode. Browser zoom changes the CSS viewport after map creation, but the old resize path only changed Leaflet's minimum zoom.

Implementation: choose a power-of-two grid tile size that keeps the buffered visible set under 96, update both prediction and highlight layers on resize, and add the corresponding offset to the server's seed-tile level. Preserve the 64 by 64 palette image and the existing 256-target, four-request, and cache limits. A wider tile intentionally has coarser biome detail at extreme browser zoom while covering the same block coordinates.

Verification: `cargo xtask frontend-check` passed typecheck, zero-warning lint, 303 unit tests and bundling. Thirty-five affected Minecraft Chromium cases passed. A focused rerun moved the 7,200 by 4,050 CSS-pixel viewport to map zoom zero, where the old 256-pixel grid required roughly 589 buffered tiles; every new 1,024-pixel prediction canvas loaded under the 96-tile target, then the grid returned to 256 pixels after resize. The public map package passed typecheck, 112 unit tests, bundling and declaration generation; exact adapter copies and source hashes were checked. No live website test or service action was performed.

Remaining: website deployment is separate. No Minecraft service stop, restart, reload, or signal is authorized.
