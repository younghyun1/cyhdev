# Site refresh: About sections, motion, palette, and dependencies

- Status: complete, 2026-11-02. About Me and About Blog are expanded into readable localized subpages from professional records; the visual system gained accent presets, a glass header, and cheap motion; the site gained a command palette, post outlines, and a home engineering showcase; dependencies are upgraded. Local only; nothing is pushed or deployed.
- Checkout: `main`, five commits ahead of `origin/main` at `77df4f5`, none behind: dependency upgrade `ae8f9e3`, tooling fix `a1b60f3`, test fix `2d9dc8a`, feature `0837012`, then this handoff. Nothing is pushed. The untracked `minecraft-20260921_052801.tar.zst` archive predates this work and is untouched.
- Completed: see [Scope](#scope) and [Decisions](#decisions).
- Verification: see [Verification](#verification).
- Remaining: owner review of translated copy and of the Soundpatrol wording, then the normal deployment workflow. WebKit was not run for this change.

## Scope

- **About Me**: overview (introduction, selected results, toolbox, section cards, contact), experience (software roles as a timeline, then work before software), projects (featured open-source cards, more work, approach), and background (education, research, awards, languages, interests). Routes `/about` and `/about/:page`.
- **About Blog**: overview (at-a-glance figures, animated request path, chapters, how changes are checked, repository source map), infrastructure, backend, frontend, subsystems (live chat SFU, Minecraft explorer, Geo-IP, WebAssembly, photography), and performance (PGO and BOLT method, the client-limited benchmark correction, setup, results, `TCP_NODELAY`). Routes `/about-blog` and `/about-blog/:page`.
- **Visual system**: accent presets (amber, lagoon, cobalt, orchid, graphite), derived glow, wash, grid, and glass tokens, a translucent header, a View Transition theme reveal, scroll-driven reveals and reading progress, CSS count-up statistics, a WebGL2 contour field behind the home hero, a home "Engineering notes" grid, and a 404 backdrop.
- **Features**: Ctrl or Command+K command palette over pages, About subpages, theme, and accent; accent swatches in the mobile drawer; blog post contents list from `h2`/`h3`, reading time, and progress bar.
- **Dependencies**: Solid, web, and signals RC.20, router next.43, Vite plugin next.55, testing-library RC.2, leaflet-geosearch 5, Vite 8.3.5, Vitest 5.1, Tailwind 4.3.4, and other minor releases; minimatch override to 11 for an eslint-chain advisory; `@types/node` 24 added because the optimization typecheck required it and nothing installed it; openidconnect 4.1 and a Cargo lockfile refresh; unused `chrono-tz` and `bigdecimal` removed; `actions/setup-node` pinned to v7.1.0.

## Content sources and exclusions

About Me draws on the September 2026 master CV and its source checklist in the private archive, the public GitHub profile README, and the previous site copy. The CV's own rule applies: confidential work stays in the private master. Soundpatrol is therefore described at the level of responsibilities, without internal systems, figures, vendors, incidents, or the data-collection work previously removed from the site. Earlier employers keep previously public figures. Excluded entirely: phone number, street-level location, certificate codes, finances, family, immigration, personal life, and the activism topics that a browser test guards against; a unit test now checks every locale for those strings.

About Blog draws on `docs/architecture/be/`, the [PGO and BOLT results](2026-09-30-pgo-bolt-results.md), the Geo-IP article draft (`blog_1.md`), and current code. Two draft claims were corrected before translation: gitleaks runs locally, not in CI, and the tantivy index covers titles and tags and is persisted on disk. The PostgreSQL 18 release month is corrected to September 2025.

Korean organization names were checked: the maritime institute is rendered as 해양수산과학기술진흥원 to match the CV's KIMST abbreviation, and Coupang as 쿠팡. The English institute name follows the CV.

## Decisions

- About pages are typed data modules per locale (`src/pages/about-locales/`) rendered by `components/content/`, replacing sixteen hand-written page components. A parity test compares every locale with English: section ids, block kinds, link targets, card links, counts, bars, tag items, list sizes, and the stat count/value agreement.
- `/about/:page` and `/about-blog/:page` use router `matchFilters`, so unknown names reach the 404 route. PGO training coverage lists both patterns, with fixed fixture paths `/about/experience` and `/about-blog/performance`.
- Accent presets replace the single-accent policy; amber remains default and unchanged. Light presets keep at least 4.5:1 contrast on paper, surface, and soft-chip grounds.
- Motion follows the [design system](../design/fe/design-system.md#motion): transform-only content animation, CSS scroll timelines behind `@supports`, explicit removal under reduced motion, and a shader that loads after idle on capable desktops only and freezes when slow.
- The command palette is a lazy chunk mounted by a small host; its shortcut is ignored inside other text fields.
- 48 UI keys were added to all eight catalogs, the browser key union, and the backend registry. The `about.nav.*` and `about_blog.nav.*` values repeat each module's page titles.
- The initial frontend graph grew from 118.9 to 125.9 KiB gzip against the 130 KiB budget; the palette, About content, and shader stay in lazy chunks.

## Verification

Final gates after the last code change, 2026-11-03: `cargo xtask frontend-check` passed (typecheck, zero-warning lint, 343 Vitest tests in 51 files, 17 optimization tests, bundle with a 125.9 KiB gzip initial graph); `cargo xtask clippy` passed for native and WebAssembly targets; `cargo xtask fmt` passed; `cargo xtask unit` passed 501 tests with no failures; `cargo xtask openapi` reported no contract drift. `npm audit` reports zero vulnerabilities.

Browser checks, Chromium only: the full `npm run test:e2e:chromium` suite passed 270 tests with one expected skip, including the new About tab and pager test, the Korean background page test, the command palette keyboard and accent persistence test, mobile reflow at 320, 390, and 844 pixels in both locales and themes, and axe checks. `npm run test:e2e:security -- --project=chromium` passed 38 tests against the built bundle under the backend policy after the fixture hash update. A temporary capture spec (since deleted) confirmed that the WebGL field starts on a 1440-pixel desktop, follows theme and accent changes, and logs no console errors; full-page captures of every new page in both themes at desktop and mobile widths were reviewed by eye. WebKit was not run; its local launch previously needed temporary system libraries.

Earlier in the session, the dependency commit passed typecheck, lint, 305 Vitest tests, Clippy, and 501 Rust unit tests before any feature work.

Corrections recorded during the work: a scripted CSS edit truncated `content-page.css` because a selector appeared twice; the tail was restored from the authored source and verified in captures. The first `frontend-check` exposed two existing problems, fixed in their own commits: the optimization typecheck lacked `@types/node`, and the seed tile fixture read the clock twice.

## Restart

`git log --oneline origin/main..main` lists the local commits. To review visually, run `npm --prefix solid-csr-spa-template ci` and `cargo xtask frontend`, then open `/about`, `/about-blog/performance`, and press Ctrl+K. Deployment follows the normal backend-embedding build; no release build was run here.