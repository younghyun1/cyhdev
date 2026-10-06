# Frontend dependency upgrade

- Status: active, 2026-10-05. Upgrade all direct frontend dependencies and refresh transitive resolutions in the website and reusable map package.
- Checkout: `main`, website base `6c39158`, map workspace base `87bab33`. Unrelated Rust manifests/lockfile and the optimized-build measurement plan are preserved.
- Completed: queried the npm registry for every direct dependency and verified Solid release-channel compatibility. The Solid 2 runtime, signals, web, and compiler target RC.13; router targets next.35 and Vite plugin next.47. Stable packages use their current latest release. TypeScript 6 remains the newest supported JS compiler for typescript-eslint; the separate native TypeScript 7 alias is already current. Leaflet stays on latest stable 1.9.4 rather than its incompatible 2.0 alpha.
- Verification: pending locked installs, frontend checks, audit, and Chromium regressions.
- Remaining: apply upgrades, fix compatibility issues, synchronize the adapter dependency reference and provenance, verify, and commit. No deployment is scoped.

Registry metadata and [Solid releases](https://github.com/solidjs/solid/releases) were consulted on 2026-10-05. The project already uses Solid 2 prereleases; runtime and compiler upgrades stay coordinated.
