# Minecraft map control and scale layout

Status: complete, 2026-09-28. The zoom control aligns with the dimension selector, and a longer distance ruler sits at the map's bottom center.

Checkout: `main` at implementation commit `555a948`, ahead of fetched `origin/main` at `cda5b79`. The scoped changes are in the Minecraft map controls, CSS, tests, and design documentation. Sillok objective: `01a0eb8c-e6b0-7dd0-aaf0-fe73eb520f29`.

Design: keep Leaflet's native zoom and Home controls. Position the top-left control corner at the same inset as the dimension selector. Render the scale in Leaflet's bottom-left corner, centered with CSS, and size it from the map viewport. Show five quarter ticks with metre or kilometre labels; limit mobile labels to avoid collisions. Preserve one block per metre at every supported zoom and dimension.

Verification: focused control tests passed (7), all four Chromium navigation and scale cases passed, and `cargo xtask frontend-check` passed typechecking, zero-warning lint, 301 tests and Vite bundling. Mobile and desktop screenshots show aligned top controls, a centered scale, and clearance from the coordinate readout and legend. The public reusable UI check passed typechecking, 110 tests and bundling. Source adapter files, reusable package source hashes and adapted documentation match their manifests; `git diff --check` passed in both repositories. No release build, deployment, or Minecraft service action occurred.

Publication: public integrations commit [`3eb70a8`](https://github.com/younghyun1/cyh_minecraft_plugins/commit/3eb70a8) is pushed to `younghyun1/cyh_minecraft_plugins`. The website implementation remains local pending its separate deployment workflow. No code work remains.
