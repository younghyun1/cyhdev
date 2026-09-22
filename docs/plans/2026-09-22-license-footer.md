# Code license and copyright footer

- Status: complete; 2026-09-22. Added a standard permissive code license and responsive footer attribution.
- Checkout: `main`, starting from `9fc10b8`, synchronized with `origin/main`.
- Decision: MIT permits reuse and derivatives while requiring preservation of copyright and permission notices; it does not mandate visible website credit. The frontend already declares MIT. Repository history starts in 2025, so the notice covers 2025-2026. Preserve third-party licenses and distinguish code from personal site content.
- Sources: [OSI MIT license](https://opensource.org/license/mit), consulted 2026-09-22.
- Implementation: add root license and README scope; add a compact copyright row without changing the mobile status interaction; cover desktop and mobile layouts.
- Verification: `cargo xtask frontend-check` passed typecheck, lint, 118 unit tests, and bundling. `cargo xtask clippy` passed native and WASM checks with existing unused-dependency manifest warnings. `npm --prefix solid-csr-spa-template run test:e2e:chromium -- e2e/page-polish.spec.ts e2e/mobile.spec.ts --grep 'search alignment|form focus|shared system status|WCAG' --workers=4 --output=../target/license-footer-tests` passed six checks, including 320/390/1440px footer layout and mobile accessibility. Reviewed screenshots at 320px and 1440px. `cargo xtask secret-scan` and `git diff --check` passed. No open PR exists for `main`.
- Correction: the first typecheck caught the existing clock signal's nullable initial value; the copyright year now initializes directly from the UTC date instead of subscribing to that clock. The full frontend gate passed after correction.
- Remaining: no implementation work. Not deployed; no release build or live-service changes. Third-party license compliance is not a complete dependency audit.
