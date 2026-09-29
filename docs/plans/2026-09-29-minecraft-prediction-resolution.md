# Minecraft prediction resolution

Status: complete, 2026-09-29. Restore the original 256 CSS-pixel prediction tiles at every browser zoom while allowing large viewports to load fully.

Checkout: website implementation is committed on `main` at `af591ae`, based on fetched remote head `9a058d6`. Public integrations started at remote head `037858a`; the adapter and reusable package mirror `af591ae` and record source hashes.

Correction: enlarging the grid to 1,024 or 2,048 CSS pixels retained only 64 by 64 samples and visibly reduced biome detail. Browser zoom must not change the seed level for a fixed map zoom.

Implementation: release retained grids after large-view movement settles, reusing warm cached tiles to retry bounded admission. Compute queue priorities once per scheduling pass and skip sorting when all four request slots are occupied. Remove viewport-driven seed grid resizing from predictions and highlights. Admit at most 2,048 active targets; preserve four concurrent requests, permission expiry and cancellation. The retained tile budget starts at 16 MiB and scales by 128 KiB per active target, capped at 256 MiB. The worst validated tile costs 116,736 bytes including two RGBA bitmap reserves, below the per-target reservation. Each prediction/highlight layer has at most 2,048 small canvases; two RGBA reserves cost 64 MiB per layer. The combined tile and two-layer image reserves are bounded by 384 MiB, excluding DOM/object/driver overhead. These are browser bounds; the existing 512 MiB seed server cache is unchanged.

Verification: `cargo xtask frontend-check` passed typecheck, zero-warning lint, 304 unit tests and bundling. `npm --prefix packages/map-ui run check` passed typecheck, 113 unit tests, bundling and declaration generation. Eighteen affected Chromium tests passed, including 7,200 by 4,050 and 10,800 by 6,085 CSS-pixel viewports at map overview, complete prediction/highlight coverage, fixed 256px tile width, unchanged seed levels, panning and resize back to normal. Large-viewport tests freeze permission timestamps to isolate geometry and admission from Playwright IPC overhead; real permission expiry remains covered by the separate expiry regression. A worst-case 500-tile palette/bitmap test proves that all tiles stay retained with no repeated reads and within the active-target cache budget. No live-site or Minecraft runtime test was performed.

Remaining: website rebuild and deployment are separate. No Minecraft service action is required or authorized.
