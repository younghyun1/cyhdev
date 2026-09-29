# Minecraft biome highlights

Status: complete, 2026-09-29. A checkbox picker highlights multiple selected biomes on permitted prediction tiles and observed survey cells.

Checkout: website `main` at implementation commits `057b8e1` and `88a332c`, ahead of fetched `origin/main` at `cda5b79`. Public integrations are pushed at [`14d4612`](https://github.com/younghyun1/cyh_minecraft_plugins/commit/14d4612). Sillok objective: `01a0ebf6-edd3-7af2-8e21-d950c8a6bc22`.

Design: offer the pinned 26.3 sampler's dimension-specific biome list in the Layers menu. Draw translucent highlights over permitted prediction tiles, sharing the existing tile store so checkbox changes do not need new seed requests. Highlight observed biome cells from the existing manual survey as well. Keep empty selections free of extra tile canvases, clear highlights on dimension changes, and leave map gestures and markers interactive.

Verification: `cargo xtask frontend-check` passed typecheck, zero-warning lint, 302 unit tests and bundling. All 43 Minecraft Chromium cases passed after moving the mobile inspection panel above the coordinate readout; the initial 42/43 run exposed that existing overlap. New checks cover multi-selection, clearing, dimension choices, cached binary and PNG tiles, zoom and observed survey cells. Desktop and mobile highlight screenshots were inspected. Public map UI passed typechecking, 111 tests, bundling and declaration generation after an explicit overlay return type corrected TS4094. Adapter files, reusable source hashes and adapted documentation match their manifests; `git diff --check` passed. No Minecraft service action or website deployment occurred.

Remaining: no implementation or public source work. Website deployment remains a separate step; no live service was changed.
