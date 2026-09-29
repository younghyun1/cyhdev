# Minecraft map control and scale layout

Status: active, 2026-09-28. Align the zoom control with the dimension selector and place a longer distance ruler at the map's bottom center.

Checkout: `main` at `cda5b79`, equal to fetched `origin/main` before edits. The scoped changes are in the Minecraft map controls, CSS, tests, and design documentation. Sillok objective: `01a0eb8c-e6b0-7dd0-aaf0-fe73eb520f29`.

Design: keep Leaflet's native zoom and Home controls. Position the top-left control corner at the same inset as the dimension selector. Render the scale in Leaflet's bottom-left corner, centered with CSS, and size it from the map viewport. Show five quarter ticks with metre or kilometre labels; limit mobile labels to avoid collisions. Preserve one block per metre at every supported zoom and dimension.

Verification: focused unit and Chromium navigation checks have passed. Mobile and desktop screenshots show aligned top controls, a centered scale, and clearance from the coordinate readout and legend. Run the complete frontend gate, synchronize the public reusable UI and host adapter, then record exact publication status. No release build, deployment, or Minecraft service action is in scope.

Remaining: finish frontend verification, public source synchronization, documentation provenance, and commits. Next command: `cargo xtask frontend-check`.
