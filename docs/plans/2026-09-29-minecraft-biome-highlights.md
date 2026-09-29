# Minecraft biome highlights

Status: active, 2026-09-29. Replace the surveyed-area single-biome filter with a checkbox picker that highlights multiple selected biomes on the map.

Checkout: `main` at `62ffad8`, clean and ahead of fetched `origin/main` at `cda5b79` by two commits before work. Sillok objective: `01a0ebf6-edd3-7af2-8e21-d950c8a6bc22`.

Design: offer the pinned 26.3 sampler's dimension-specific biome list in the Layers menu. Draw translucent highlights over permitted prediction tiles, sharing the existing tile store so checkbox changes do not need new seed requests. Highlight observed biome cells from the existing manual survey as well. Keep empty selections free of extra tile canvases, clear highlights on dimension changes, and leave map gestures and markers interactive.

Verification: `cargo xtask frontend-check` passed typecheck, zero-warning lint, 302 unit tests and bundling. All 43 Minecraft Chromium cases passed after moving the mobile inspection panel above the coordinate readout; the initial 42/43 run exposed that existing overlap. New checks cover multi-selection, clearing, dimension choices, cached binary and PNG tiles, zoom and observed survey cells. Desktop and mobile highlight screenshots were inspected. Synchronize the public reusable UI and website adapter, verify source manifests, then publish the public integration source. No Minecraft service action or website deployment is in scope.

Remaining: implement, verify, sync, and commit. Next file: `solid-csr-spa-template/src/components/minecraft/seedTileLayer.ts`.
