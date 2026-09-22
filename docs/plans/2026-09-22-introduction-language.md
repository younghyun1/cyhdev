# Introduction language consistency

- Status: complete; corrected the Korean Home introduction and kept About Me/About Blog content English pending complete translations.
- Checkout: `main`; initial implementation `f5e3f44`, followed by a Korean wording refinement.
- Cause: Korean Home role, summary, and principle values contain English in both source catalogs; About Me switches only projects, volunteering, and hobbies to Korean.
- Scope: translate the three Home entries in both catalogs; remove partial About translation branches; declare English content language on both About pages. Preserve global language selection and existing dynamic catalog loading. Future About translations should cover whole pages rather than isolated sections.
- Verification: `cargo xtask frontend-check` passed typecheck, lint, 118 unit tests, and bundling. `cargo xtask clippy` passed native/WASM checks with existing unused-dependency manifest warnings. `cargo xtask secret-scan` and `git diff --check` passed. `npm --prefix solid-csr-spa-template run test:e2e:chromium -- e2e/localization.spec.ts e2e/page-polish.spec.ts --workers=4 --output=../target/introduction-language-tests` passed 22 browser checks. No open PR exists for `main`.
- Correction: the initial new browser test reset English on each reload, causing one test failure; changed its setup to initialize only a missing preference and reran both suites successfully.
- Wording refinement: the Korean principle now reads “장인 정신을 가지고 작업하는 것을 지향합니다.” in both catalogs, as requested. Repeated frontend checks (118 tests), Clippy, confidential-information scan, and the focused Chromium test selected by `--grep 'Korean Home'` passed. Other languages are unchanged.
- Remaining: no scoped implementation work. Complete About translations are deferred by request. No deployment, release build, or live database mutation; updated backend catalogs take effect through normal startup synchronization.
