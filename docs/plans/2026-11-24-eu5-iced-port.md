# Port the EU5 Location Filter from Slint to iced

- Status: active, 2026-11-24. Intended outcome: the vendored EU5 Location Filter runs on iced 0.14 on desktop and in the browser, on EU5 1.4 data, following the site's palette, language, direction and motion.
- Checkout: parent branch `feat/eu5-iced` from `096b2ea2`; submodule `vendor/eu5-location-filter` branch `feat/iced-port` from `2499f0f`.
- Completed: scope and design decisions below.
- Verification: none yet; no code has been compiled for this work.
- Remaining: contracts wave, implementation wave, review wave, then one integration pass (compile, test, 1.4 import, language and font packs, browser pass, measurements). Next action: write the frozen module skeleton in the submodule.

## Context

`vendor/eu5-location-filter` (own repository, pinned submodule at `2499f0f`) is a Slint 1.17 desktop and browser app over 28,573 static EU5 locations. Its core (`filter/`, `model/`, `storage/`, `index_storage`, `import/`, `parser/`, `steam`, `embedded`) is UI-agnostic and already fast (indexed queries 0.02-2 ms in the dev profile). The UI layer is 1,141 lines of `.slint` plus about 60 KB of glue in `src/ui/`. In cyhdev it runs in a sandboxed opaque-origin iframe that only receives `light` or `dark`.

Goal: replace the UI with iced, held to explicit performance budgets, redesigned to follow the site (full palette, language, direction, motion, narrow screens, shareable URLs), with data updated to the EU5 1.4 install on `/run/media/cyh/OS` (Steam build 26547371).

## Decisions taken

| Topic | Decision |
| --- | --- |
| Targets | Desktop and browser |
| Migration | Replace Slint on a submodule branch; no side-by-side feature |
| iced | 0.14.0 from crates.io (latest release, 2025-12-07) |
| Browser renderer | wgpu WebGPU with the `webgl` fallback feature |
| Result table | One custom widget owning header and body |
| Look | Free redesign that follows the site |
| Accessibility | No screen-reader tree in iced 0.14; accepted and documented |
| UI strings | App-owned catalogs for all 27 site locales |
| Game text | English embedded; 10 lazy language packs with their own search and sort indexes |
| Fonts | IBM Plex embedded for Latin and Cyrillic; lazy per-script packs, Noto where Plex has no face |
| RTL | Mirror the layout for `ar`, `ur-PK`, `fa-IR` |
| Site integration | Full palette sync, reduced motion, narrow and touch layout, deep links |
| Embedding | Keep the sandboxed frame; versioned message protocol |
| Data | Re-import 1.4 plus an audited 1.3.11 to 1.4 diff, landed before the port |
| Performance proof | Release builds allowed inside the submodule only; parent no-release rule unchanged |
| Delivery | Spike with a go/no-go checkpoint, then the full port |

## Facts verified in source

- iced 0.14 on wasm32 replaces the DOM node named by `window::Settings.platform_specific.target` with its own canvas (`winit/src/lib.rs`), so `target = "canvas"` keeps `web/index.html` usable; the new canvas has no id, and the existing `canvas` CSS selector still applies.
- The stock `table` widget builds every cell eagerly and no stock widget virtualizes; `scrollable` exposes `on_scroll(Viewport)`.
- `graphics/src/text/cache.rs` trims entries unused since the previous frame, so drawing only visible cells keeps the text cache bounded.
- 0.14 has reactive rendering, input-method support, headless `iced_test` (`Simulator`, snapshots), and no accessibility integration.
- EU5 files serve with `Access-Control-Allow-Origin: *` (`browser_policy.rs`), so the frame can fetch extra assets from its own directory.
- The 1.4 install has every file `steam.rs` requires and `localization/<language>/location_names/` for 11 languages.

- 0.14 `core/src/text.rs` has `Wrapping` but no ellipsis, so narrowed cells clip; any ellipsis is local code.
- `steam::validate_explicit` reads `appmanifest_3450310.acf` two levels above a `steamapps/common/<game>` path, so `--game-dir` on the mounted drive yields the real build ID.
- cyhdev embeds the frontend `dist/` into the backend binary (`static_assets.rs`), and the Dockerfile copies exactly three EU5 files and keeps only `.gz` variants.

Not yet verified, so each is a spike item: cosmic-text font fallback on wasm, Korean IME in the browser search field, iced or winit touching storage or clipboard at startup inside the sandbox, a layout-preserving font subsetter. Four spike items.

## Performance budgets (proposed figures; optimized builds, 1440x900, default Land result of about 21,000 rows)

- Table scroll, column resize, pane drag: update + layout + draw CPU p95 at most 4 ms native and 8 ms in the browser; cost independent of row count.
- Search or filter change to presented frame: within one frame; no per-row allocation outside the visible window.
- First paint before bundle decode on both targets; decode never blocks a painted frame on desktop.
- Optimized `.wasm`, raw and gzip: compared against the Slint build of the same data. wgpu with WebGL will likely exceed femtovg; the measured number goes to the checkpoint instead of being assumed.

## Architecture

**Unchanged core.** Only `src/ui/`, `ui/*.slint`, `build.rs` (drop `slint_build`, keep bundle staging), Cargo features, and `web/index.html` change shape. `FilterEngine::apply` stays synchronous inside `update`.

**Module layout, replacing `src/ui/` with `src/app/`:**
- `app.rs`: state, `Message`, `update`, `view`, subscriptions.
- `view_model/`: `columns.rs`, `filter_selection.rs`, `options.rs`, `detail.rs`, `row.rs`, ported from the existing files with Slint types removed (plain `Vec`, `&str`, typed enums instead of string field keys). Existing tests move with them.
- `widget/window.rs`: fixed-row-height windowing math, scrollbar geometry, hit testing, direction mirroring; pure and unit tested.
- `widget/result_grid.rs`: the table. Borrows `&[LocationId]`, `&Dataset`, columns, sort and selection; keeps scroll offsets, hover and drag state in widget state; draws quads and clipped text only for visible rows and columns; symbol cells borrow dictionary strings, numeric cells format into one reused buffer. Handles wheel, Shift-wheel, middle-button pan, touch drag, row click, header sort click, header-edge resize, and its own scrollbars. Emits `RowSelected`, `SortRequested`, `ColumnResized`. Focusable: arrows, Page Up/Down, Home/End move the selection with scroll-into-view; Tab leaves the grid.
- `widget/checklist.rs`: virtualized searchable checkbox list on the same windowing core (province, area and culture lists run to thousands of options). Option labels are folded once per language, not per keystroke as `options::filtered` does today. Focusable: arrows move the active option, Space toggles it, so keyboard operation survives the loss of stock checkboxes.
- `panes/`: header, filter rail, detail, column menu, about, loading and error overlays, built from stock iced widgets.
- `theme.rs`: `Palette` of site tokens and the custom style functions; 90 ms mix on change unless reduced motion.
- `layout.rs`: `Direction`, direction-aware row and alignment helpers, breakpoints.
- `i18n.rs`: generated `Text` key enum, per-locale static tables, plural selection, locale to game-language mapping.
- `load.rs`: desktop decode on a dedicated thread reporting through a channel (existing `worker.rs` logic, including retry and rebuild); browser paints the shell and progress first, then decodes.
- `host.rs` (wasm only): `wasm-bindgen` exports feeding a bounded latest-wins channel read by a `Subscription`.

**Host protocol v2.** The parent page posts one object `{type: "cyhdev:eu5-host", v: 2, theme: {mode, paper, surface, ink, inkMuted, inkFaint, line, lineStrong, accent, accentStrong, accentSoft, ok, danger}, locale, dir, reducedMotion, viewState?}` on ready and on any change, with colours resolved to sRGB from computed styles. `web/index.html` checks source window, type, version, size and field types, then calls typed exports (`set_palette`, `set_locale`, `set_reduced_motion`, `apply_view_state`); Rust validates again. The app reports a debounced, bounded view-state string (search, facet selections by symbol key, ranges, sort, selected location key) through `cyhdev:eu5-state`; the parent validates and mirrors it into the URL with history replace, and sends it back on load. The v1 string messages are removed; both sides change in the same parent commit as the gitlink. On desktop the same state has in-app language and theme controls, hidden in the browser.

**Localization.**
- UI catalogs: one key-value file per locale under `assets/i18n/`, compiled by `build.rs` into static tables indexed by the generated enum; English is the fallback. A test enforces key, placeholder and plural-category parity for all 27 locales. English and Korean are written directly; the other 25 are drafted and listed in the PR as needing review.
- Game text: `GameInstallation::localization_roots` takes a language; `import` also writes `assets/lang/<language>.bitcode.zst` holding localized labels, search strings (localized name, English name, key), trigram postings, and the label-dependent sort orders and ranks. The index schema splits language-independent parts (facets, numeric, non-label orders) from the language part, with a version bump and the same validation and scan-equivalence tests per language. Browser fetches a pack on selection; desktop embeds all. Pack file names carry the build ID and the app rejects a pack whose header build or location count differs from the embedded dataset, so a cached pack cannot pair with a newer wasm. One non-English pack is held in memory at a time. Site locales without game text use English.
- Fonts: subset IBM Plex Sans and Mono (Latin, Latin Extended, Cyrillic) embedded. Lazy packs under `assets/fonts/`: Plex Sans KR and JP and a Simplified Chinese face subset to the glyphs the catalogs and name packs use; Noto Sans Arabic, Devanagari, Bengali and Telugu shipped whole because shaping tables must survive. A Rust subcommand generates packs from local source fonts and records source version and SHA-256; OFL texts are added and shown in About.
- Coverage: a generated test checks that every codepoint in each catalog and name pack exists in the cmap of the font assigned below.
- RTL: `Direction` reverses pane order, alignment, the grid's column origin and scrollbar side, and splitter drag sign.

| Site locales | Font | Game text | Direction |
| --- | --- | --- | --- |
| en-US | embedded Plex | english | LTR |
| fr-FR, de-DE, es-ES, pt-BR, pl-PL, tr-TR | embedded Plex | french, german, spanish, braz_por, polish, turkish | LTR |
| ru-RU | embedded Plex (Cyrillic) | russian | LTR |
| it-IT, sv-SE, nl-NL, ro-RO, id-ID, tl-PH, sh, vi-VN | embedded Plex (subset must include Vietnamese) | english | LTR |
| uk-UA | embedded Plex (Cyrillic) | english | LTR |
| ko-KR | Plex Sans KR pack | korean | LTR |
| ja-JP | Plex Sans JP pack | japanese | LTR |
| zh-Hans | Simplified Chinese pack | simp_chinese | LTR |
| zh-Hant | Traditional Chinese pack (catalog glyphs only) | english | LTR |
| ar, fa-IR, ur-PK | Noto Sans Arabic pack | english | RTL |
| hi-IN | Noto Sans Devanagari pack | english | LTR |
| bn-BD | Noto Sans Bengali pack | english | LTR |
| te-IN | Noto Sans Telugu pack | english | LTR |

**Untrusted input.** Host messages, the view-state string and fetched packs are parsed with explicit bounds and rejection paths, unit tested on malformed and oversized input, with `fuzztest` targets for the view-state and pack decoders.

**Conventions.** `rust-conventions` applies to new code: explicit `match` on every `Option` and `Result`, no `unwrap` or `expect` (iced's own internal panics are outside this crate), files under 300 lines, `mod.rs` holding only declarations, rustdoc on modules and functions. The crate keeps its existing allocator and toolchain choices; none of that changes incidentally.

**Narrow and touch.** Below a width breakpoint the filter rail and detail pane become overlay drawers and the header stacks; the browser build drops the 1050 px minimum. The grid and checklists scroll by touch.

## Stages

Each stage is a commit series on a submodule branch; the parent gitlink advances only with the matching parent changes.

1. **Setup.** Load `rust-conventions`, `frontend-conventions` and the scoped `AGENTS.md` files; fetch and confirm both checkouts at their remotes' heads; parent work goes on `feat/eu5-iced`; create `docs/plans/2026-11-24-eu5-iced-port.md` in the parent with these decisions; open the Sillok objective; branch the submodule from `2499f0f`.
2. **EU5 1.4 data on the Slint UI.** `cargo run --locked -- import --force --game-dir "/run/media/cyh/OS/Program Files (x86)/Steam/steamapps/common/Europa Universalis V"` (read-only use of the mount). The import must report build 26547371. Fix parser breaks. Extract all 11 languages now, or copy the needed `map_data`, `common` and `localization` inputs to a snapshot directory outside both repositories, so a later Steam update cannot change inputs between this stage and stage 7; stage 7 asserts the same build. Add a `diff` subcommand comparing two bundles by location key and field; review the 1.3.11 to 1.4 report and the population-capacity inputs. Take the exact version string from the install rather than assuming it; update `EU5_GAME_VERSION`, the hardcoded `1.3.11` strings in `worker.rs` and `web.rs`, expected counts in `embedded.rs` tests, README, docs and CHANGELOG. Record Slint baselines: optimized wasm size raw and gzip, startup. Submodule PR, then gitlink bump. The owner merges submodule PRs; the gitlink only ever points at pushed submodule commits.
3. **Spike and checkpoint.** iced shell, load path, `result_grid` on real data, native and browser inside the sandboxed frame. Resolve the four unverified items, measure against the budgets with and without `webgl`, and stop for a go/no-go decision with the numbers.
4. **Core port.** View-model modules, panes, checklist, dialogs, desktop retry and rebuild; delete `ui/*.slint`, `slint` and `slint-build`; update features (`desktop`, `web`, `web-benchmark`), CI Linux packages and the wasm job.
5. **Host protocol v2 and deep links**, with the parent changes below.
6. **UI catalogs, fonts, RTL.**
7. **Game language packs.**
8. **Narrow and touch layout.**
9. **Performance pass and close-out.** Measure, fix, record results; update submodule `README.md`, `CHANGELOG.md`, `docs/development.md`, `docs/design/fe/layout.md`, `docs/data-format.md`; PRs through `gh` in the documented format; final gitlink.

## Parent repository changes

- `solid-csr-spa-template/src/pages/eu5_locations_db.tsx` and `src/__tests__/eu5_locations_db.test.tsx`: protocol v2 sender (palette from computed styles, `locale()`, direction, motion), state receiver with validation, URL mirroring.
- `solid-csr-spa-template/src/styles/eu5-locations-db.css`: page background from site tokens.
- `home.showcase.eu5.kicker` and text ("Slint + WASM") in `src/i18n/defaults/`, `rust-be-template/i18n/ui/*.json` and the backend key source.
- `tools/xtask/src/eu5_web.rs` and its tests: stage `lang/` and `fonts/`; `rust-be-template/Dockerfile`: copy them, keep already-compressed packs out of the second gzip pass if the asset server allows a stored variant, and match `WASM_BINDGEN_VERSION` to the new `Cargo.lock`.
- `rust-be-template/src/routers/main_router/static_assets.rs` tests: nested asset paths, content types and the CORS header for packs.
- `docs/design/fe/` gains the frame protocol and deep-link description; `docs/design/fe/site-map.md`, `vendor/AGENTS.md` (Slint references), the plan document.
- No generated client or HTTP contract changes.

## Verification

Submodule: `cargo fmt --check`; `cargo clippy --locked --all-targets -- -D warnings`; `cargo clippy --locked --target wasm32-unknown-unknown --no-default-features --features web -- -D warnings`; `cargo test --locked --all-targets`; ignored timing and regeneration tests with `--ignored --test-threads=1`; `cargo doc --no-deps`. New tests: windowing math including mirrored and clamped cases; ported view-model tests; `iced_test` simulator flows (search, facet toggle, sort, column toggle, clear, drawer layout, RTL snapshot); catalog parity; per-language pack validation and scan equivalence; protocol and view-state round trips with oversized and malformed input.

Performance, submodule only: a headless harness scripts scroll, resize and typing against the real bundles and reports median and p95 per phase; `wasm-pack build --release` for size; the `web-benchmark` export reports browser frame intervals. Results are recorded next to the existing timing table with toolchain, commit and machine.

Parent: `cargo xtask eu5-web-stage`, `cargo xtask clippy`, `cargo xtask fmt`, `cargo xtask unit`, `cargo xtask frontend-check`, `npm --prefix solid-csr-spa-template run test:e2e:chromium`, `test:e2e:security` and `test:e2e:layout`. Manual browser pass on the staged app: theme and accent change, each script family, an RTL locale, Korean IME input, a deep link reload, a narrow viewport, WebGPU on and off. `cargo xtask image-smoke` is run only if it exercises the EU5 stage (checked first), as the one permitted gate for the Dockerfile change. `cargo xtask build`, `image`, `wasm-build` and `final-review` stay unrun; the optimized image path is reported as unverified.

## Risks

- Download size and the WebGL fallback pull in opposite directions; decided at the checkpoint from measurements.
- Browser IME, wasm font fallback and ellipsis may need local workarounds or a pinned newer iced; the spike settles this before the Slint UI is deleted.
- Twenty-five drafted catalogs need human review before they are presented as finished.
- Language packs (estimated 1-2 MB compressed each) and font packs grow the submodule repository, the desktop binary, and the cyhdev backend binary and image, because the site embeds the staged app; sizes are measured in stage 7 before the gitlink moves.

## Execution with tiered parallel subagents

Constraint set for this run: subagents write code and prose only. They never run `cargo`, `npm`, `wasm-pack`, the importer, or any Git write. The orchestrating session compiles, tests, imports, measures and commits once, after all writing is merged. This replaces the stage 3 spike checkpoint: nothing is compiled before the full port exists, so interface drift and wrong iced API usage are the main risks and the structure below exists to contain them.

### What stays with the orchestrating session

- Before any fan-out: stage 1 setup (branches, plan document, Sillok objective) and a read-only reference copy of the exact `iced` 0.14.0, `iced_*`, `cosmic-text` and `winit` sources in the scratchpad (`cargo fetch` or registry sources; fetching is not compiling), so agents read real signatures instead of recalling them.
- Between waves: read each wave's result and the diff, resolve contract change requests, decide the next wave.
- After the last wave: everything in "Final integration" below, including all artefacts that need a running binary (1.4 bundles, language packs, font packs).

### Rules given to every subagent

- Edit only the files listed as owned; read anything. Owned sets are disjoint, so all agents share one working tree and no worktrees are used.
- Frozen contract files (wave A output) are read-only for wave B. A needed signature change is returned as a request, not made.
- No build, test, format or Git commands. Return a structured result: files written, contract change requests, assumptions that could not be checked against the reference sources.
- Check every iced call against the scratchpad reference before writing it.

### Wave A: contracts (2 agents, sequential)

| Agent | Model, effort | Output |
| --- | --- | --- |
| A1 API digest | Opus 5.5, high | Scratchpad notes with exact 0.14 signatures for `Widget`, application builder, `Task`, `Subscription`, font loading, text drawing, touch and keyboard events, `iced_test`; answers from source for wasm font fallback and startup storage or clipboard access. Browser IME cannot be answered from source and stays for final integration. |
| A2 Skeleton | Opus 5.5, high | `Cargo.toml` features and dependencies; the full `src/app/` module tree with every public type, `Message`, function signature and rustdoc, bodies as minimal placeholders; language-pack and split-index schema types; host protocol constants and the view-state grammar; the English catalog with the complete key list; the TypeScript protocol types for the parent page. |

The session reviews A2 without compiling and freezes it.

### Wave B: implementation (about 27 agents, parallel)

| Tier | Agents and owned files |
| --- | --- |
| Opus 5.5, high | B1 `widget/window.rs` and `widget/result_grid.rs`. B2 index schema split, language-pack storage and validation in `filter/`, `index_storage.rs`, `storage/`. |
| Opus 5.5, medium | B3 `widget/checklist.rs`. B4 `app.rs` update, view and subscriptions. B5 `host.rs`, the view-state codec, `web/index.html`. B6 importer language roots, pack writer, `diff` subcommand, `main.rs`, reading the 1.4 install read-only for file formats. B7 `layout.rs`: direction mirroring, breakpoints, drawers. |
| Sonnet 5.5, medium | B8 `view_model/` ports with their tests. B9 header and filter-rail panes. B10 detail, column menu, about, loading and error panes. B11 `theme.rs`. B12 `i18n.rs`, `build.rs` catalog codegen, parity test. B13 `load.rs`. B14 font-pack subcommand and cmap coverage test. B15 parent `eu5_locations_db.tsx`, its CSS and Vitest file. B16 parent `eu5_web.rs`, Dockerfile, `static_assets` tests. B17 windowing unit tests and `iced_test` simulator flows. B18 protocol, view-state and pack decoder tests and fuzz targets. |
| Sonnet 5.5, low | B19 submodule `README.md`, `CHANGELOG.md`, `docs/`. B20 parent `docs/design/fe/`, `site-map.md`, `vendor/AGENTS.md`. B21 catalogs for `ar`, `fa-IR`, `ur-PK`, `hi-IN`, `bn-BD`, `te-IN`. |
| Haiku 5.5 | B22-B25 the remaining 19 catalogs, about five locales per agent. B26 the showcase kicker and text in the parent defaults and 27 backend catalog files. B27 submodule CI workflow and the mechanical `1.3.11` string and constant updates. |

B17 and B18 write tests against the frozen contracts, not against implementations, so they run in the same wave.

### Wave C: read-only review, then fixes (4 reviewers, then up to about 8 fixers, at most 2 rounds)

- Four Opus 5.5 high reviewers, one lens each: iced API use against the reference; every call site against the frozen signatures and each other; `cfg` and feature gating for `desktop`, `web` and wasm32; conventions (explicit error handling, bounds on untrusted input, file size, rustdoc).
- Findings are deduplicated in script code and routed to one Sonnet 5.5 medium fixer per owning file set; anything touching the grid or index schema goes to Opus 5.5 high.
- A second reviewer round runs only on files changed by fixers. Remaining findings are handed to final integration, listed, not dropped.

### Final integration (orchestrating session only)

1. Review the whole diff; first compile of native and wasm targets. Expect a substantial error list from several thousand never-compiled lines; fix in place until `cargo clippy` passes for both targets, then `cargo fmt` and `cargo test`.
2. Run the 1.4 import against the mounted install, confirm build 26547371, run the bundle diff, review it, regenerate indexes and the ten language packs; obtain source fonts and generate font packs; run the ignored timing and regeneration tests.
3. Parent gates and `cargo xtask eu5-web-stage`; browser pass in the sandboxed frame including Korean IME; optimized submodule builds for the performance and size measurements, with the Slint baseline taken from the pre-port commit.
4. Commit in coherent units, push, open PRs, advance the gitlink.

### Workflow shape and cost

Three workflow runs (A, B, C) with the session reading results between them, about 33 agents plus fixers. That exceeds the default guideline of under 10 agents per workflow; wave B alone is one run of 27 at up to 16 concurrent.

### Known cost of the no-compile rule

- The go/no-go checkpoint is gone: if iced cannot meet a budget or the browser IME is unusable, that is found after the port is written.
- Bundle-dependent test expectations (location counts, capacity values for 1.4) cannot be written by subagents; they are filled in during final integration.
- Drafted catalogs from the Haiku tier are unreviewed machine translations and are labelled as such in the PR.
