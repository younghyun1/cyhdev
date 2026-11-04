# Interface pass: home board, forum, animation switch, page styling

- Status: complete, 2026-11-04. All nine interface items are implemented and the optimized build of `5d63813` reached `miniserver:~/`; the commits are pushed, but nothing is installed or restarted on the server.
- Checkout: `main`, implementation through `5d63813`.
- Completed: see [Scope](#scope) and [Decisions](#decisions).
- Verification: see [Verification](#verification).
- Remaining: owner review of the new translated strings; then the normal deployment workflow.

## Scope

- **Accent on desktop:** the header gained an appearance button showing the active accent. Its popover holds the accent swatches and the animation switch. Before this, only the mobile drawer and the command palette could change the accent.
- **Login dots:** the red and green dots are gone from the header and the drawer, along with `--glow-ok` and `--glow-danger`.
- **Animation switch:** `state/motion.ts` and `data-motion="off"` replace every `prefers-reduced-motion` query. Animations default to on regardless of the OS setting, at the owner's request. The header popover, the mobile drawer, and the command palette expose the switch.
- **Contour map:** it was not just slow; it was frozen. See [Corrections](#corrections). It now also drifts faster, its contour levels rise so lines keep migrating, and a soft swell travels through the bands. Phones and animation-off visitors get one still frame.
- **Home board:** chat sits in a sticky left column beside posts and photographs, with engineering notes last, on narrow gutters up to 128rem. Phones stack posts, photographs, chat, notes. Chat is a lazy chunk mounted near the viewport.
- **Forum:** a topic table with reply and activity columns and a pinned group; threads of numbered posts with an author column, permalinks, quoting, URL links, relative times, and collapsed moderation; linkable searches; sign-in prompts instead of disabled forms.
- **Mobile:** a shorter status bar, an account shortcut in the header, a compact home hero, three-across home photographs, two-across statistics and Geo-IP fields, and inline actions where full-width stacks wasted space.
- **Other pages:** `styles/surfaces.css` and `pageStyles` give every route the About pages' grid backdrop, title rule, and section markers. The blog list became hairline rows with summaries. Live chat became dense rows and lost its duplicate heading. Geo-IP became two columns. Projects, photographs, authentication, the public profile, and administration headings were aligned.

## Decisions

- Chat keeps one global socket provider; mounting it on the home page means each desktop home visit holds a socket while the panel is mounted. The panel scrolls only its own list, because `scrollIntoView` would have dragged the whole document to the panel on every message.
- The home chat slot has its own error boundary. Without it, a chunk that fails to load, as after a deploy replaces hashed assets, would take the whole home page down.
- The panel height on desktop leaves room for the hero, so the composer is visible on arrival. Viewports under 560px tall drop the sticky position.
- 29 UI keys were added to all eight catalogs, the browser key lists, and the backend registry; `forum.topic.back` and `forum.topic.created` became unused and were removed.
- The inline pre-paint script changed, so `e2e/security-headers.json` carries a new script hash. The backend computes its own hash from the built shell at startup.
- The optimization campaign's route assertions in `tools/optimization-fixtures/src/campaign.rs` follow the renamed topic heading and profile name.

## Corrections

The contour field's slow-device probe averaged the interval between drawn frames and stopped animating above 28ms. Frames are throttled to at least 33ms apart, so the average always exceeded the limit and the field froze after 20 frames, about 0.7 seconds, on every machine. The November 2 verification only confirmed that it started. The probe now sees every animation-frame callback and stops only below 20 per second; `contourField.test.ts` covers it.

The `/about` portrait stretched into a wide pill on phones, and the `/about-blog` tabs overlapped at 390 pixels. Both predate this work and are fixed.

## Verification

At `5d63813`:

- `cargo xtask frontend-check` passed: 354 Vitest tests, 17 optimization tests, and an initial route graph of 127.9 KiB gzip against the 130 KiB budget.
- `npx playwright test --project=chromium` from `solid-csr-spa-template/`: 285 passed, 1 skipped. This includes the new `e2e/site-interface.spec.ts`.

At `b9577bb`, whose Rust and HTTP surface `5d63813` did not change:

- `cargo xtask clippy`, `fmt`, `unit`, and `openapi` passed.
- `npm run test:e2e:security -- --project=chromium`: 38 passed.
- `cargo test --locked -p optimization-fixtures`: 6 passed.

Contour field, once, with a temporary spec that was not kept, under software GL: 60.5 animation-frame callbacks per second, 4 of 4 sampled frames distinct over 12 seconds, frozen after switching animations off, and moving again after switching them on.

Every route was reviewed by eye through contact sheets at 1920, 1440, 1024, 800, and 390 pixels against mocked APIs, in both themes where relevant.

`./build_pgo_and_bolt.sh` on a clean `5d63813` was accepted as run `cyhdev-pgo-bolt-18e4d4512a02ed27_1667d`. The live browser campaign, which checks each route against the built backend and synthetic fixture data, passed in all 14 runs: five profiling and coverage runs and nine benchmark rounds. Rounds were rotated on a host left on its balanced policy (`powersave`, `balance_performance`), so the ratios matter more than the absolute numbers:

| Round | Baseline req/s | PGO req/s | BOLT req/s |
| --- | --- | --- | --- |
| 1 | 52199 | 62607 | 63106 |
| 2 | 53212 | 61337 | 62785 |
| 3 | 53228 | 61322 | 62730 |

Median of paired rounds against baseline: PGO throughput 1.153x, p95 0.863x, p99 0.866x; BOLT throughput 1.180x, p95 0.848x, p99 0.850x.

`scp` placed `dist/rust-be-template` at `miniserver:~/rust-be-template` and `dist/minecraft-seed` at `miniserver:~/minecraft-seed`; neither path existed beforehand. `sha256sum` matched on both ends: `bbb9167d...cfe7` for the server, which is also the receipt's BOLT stage, and `58bb9d73...5c7b` for the seed helper.

Not run: WebKit projects and `test:e2e:performance`. The transferred binaries have not been started on the server, so the deployed site itself is unverified.
