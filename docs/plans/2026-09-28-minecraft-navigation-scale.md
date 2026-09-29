# Minecraft origin navigation and distance scale

Status: active, 2026-09-28. Add a compact Home control that centers the current dimension at X 0, Z 0 without changing zoom, a dynamic scale bar labelled in kilometres with one block equal to one metre, and a distinct frozen-river biome color.

Checkout: `main` at `0aec95d`, one documentation commit ahead of fetched `origin/main` at `7c876ab`. Public integration repository is clean at `475304e`. Sillok objective: `01a0eb11-1c25-7d71-99cb-36fe5c3e9e41`.

Design: extend the current Leaflet map controls and centralized Minecraft styles. Account for the existing block-to-map coordinate scale; do not use geographic distance. Keep the selected dimension and zoom, avoid map click/selection propagation, support keyboard and narrow screens, and remove listeners with the map. Update the standalone map UI and preserved host adapter together.

Verification: run focused control/browser checks and the required frontend gate. Confirm origin navigation, scale accuracy over zoom levels and configured native zooms, resize behavior, and dimension changes. Verify frozen rivers use matching icy cyan in the frontend and PNG encoder, separate from rivers, oceans and snowy terrain. The backend change is limited to display colors; no protocol change, release build, installation, deployment or service restart is authorized.

Completed: Home preserves dimension, zoom and selection while clearing stale hover. A responsive 1/2/5 kilometre scale uses the actual block-to-pixel ratio and keeps its label readable over short bars. Frozen rivers use icy cyan in binary/survey rendering and native PNG pixels. The canvas has its own stacking context so controls cannot intercept mobile menu clicks. The map fixture now tolerates its clock context being destroyed when navigation cancels an outstanding tile request. Public adapter and reusable UI/codec copies are synchronized.

Verification: `cargo xtask clippy`, `cargo xtask fmt`, `cargo xtask unit` and `cargo xtask frontend-check` passed. All 35 Minecraft Chromium regressions passed, including the four new navigation/scale cases, and the final narrow-screen screenshot was inspected. WebKit's four new cases could not launch because the host lacks `libicu74` and `libflite1`; no WebKit runtime result is claimed. Independent map-ui checks passed 104 tests, typechecking and bundle/declaration generation; extracted codec passed nine tests and strict all-target Clippy. Changes introduce no HTTP contract or persistence behavior.

Remaining: record implementation provenance, commit locally and push the verified plugin/integration changes to the public repository. Website deployment and service activation remain outside this work.
