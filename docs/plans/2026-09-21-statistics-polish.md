# Statistics visual consistency

Status: complete. Host statistics use the existing cream, black, and amber design system without changing metrics transport or retention.

Checkout: `main`, starting at `2865879`. Scope: host statistics components, chart palette, dedicated stylesheet, and focused browser coverage.

Completed: replaced gradients, heavy shadows, and literal palettes with centralized `statistics.css` and shared CSS tokens. Chart legends use existing translation keys; canvases have accessible labels. Palette observers refresh after theme-class or mobile-breakpoint changes and disconnect on disposal. The live status dot stays neutral until metrics arrive; errors use the existing danger token. Host details no longer stretch into a mostly empty full-height panel. Corrected the global mobile light-mode accent selector leaking into dark mode, restoring amber contrast. The frontend conventions skill guided centralized styling and focused unit coverage.

Verification: `npm --prefix solid-csr-spa-template run typecheck` and `npm --prefix solid-csr-spa-template run lint -- --max-warnings 0` passed. `npm --prefix solid-csr-spa-template run test -- chart_palette.test.tsx` passed one unit test covering token updates and cleanup. `npm --prefix solid-csr-spa-template run test:e2e:chromium -- statistics-polish.spec.ts --workers=2` passed six cases: 320, 390, and 1440px, light/dark. Each case checks populated binary CPU/memory values, flat one-pixel panels, canvas labels, dark accent, live theme switching, overflow, and uncaught errors. `git diff --check` passed. The combined `test:e2e` run passed Chromium but could not launch WebKit because its executable is not installed; no WebKit pass is claimed.

Renders: before captures are under ignored `target/statistics-polish/before/` for desktop/mobile light/dark. Final populated captures are under `target/statistics-polish/after/chromium/` as `<width>-<theme>.png`. Desktop light/dark and mobile dark captures were inspected at full size; plots retain visible labels and avoid horizontal overflow. These are deterministic fixtures, not live-service captures.

Remaining: none in this selected polish scope. Other suggestions remain proposals. Live embedded-page verification is dismissed and outside this work. Coordination owns the final commit and broader project verification; rerun the Chromium command above to reproduce these captures.
