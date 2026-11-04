# Interface pass: home board, forum, animation switch, page styling

- Status: complete, 2026-11-04. All nine interface items and the [follow-ups](#follow-ups) are implemented. The optimized build in `miniserver:~/` was made from `5d63813`, before the follow-ups, so it does not contain them; nothing is installed or restarted on the server.
- Checkout: `main`; see `git log --oneline origin/main..main` for anything unpushed.
- Completed: see [Scope](#scope), [Decisions](#decisions), and [Follow-ups](#follow-ups).
- Verification: see [Verification](#verification).
- Remaining: owner review of the new translated strings; a new optimized build if the follow-ups should deploy; then the normal deployment workflow.

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

## Follow-ups

Owner requests after the first pass:

- **Chat between routes:** the socket outlives a route change through a 400 ms grace period, but the server sends `hello` only once per connection, so a panel mounting on a reused socket had no history, identity, or count. `state/live_chat_socket.tsx` now keeps the connection's greeting current through `services/live_chat_greeting.ts`, bounded at the 50 messages a `hello` carries, and each new panel starts from it.
- **Chat height:** the column takes its height from the other panels and ends level with the notes. Where the panels stack and the column outgrows the screen, chat stays in view and meets the notes at the end of the page.
- **Short post lists:** five posts share the box evenly; fewer stay thin rows at the top of a box that still matches the photographs. An empty list shows its message rather than an empty box.
- **Rectified failures stay small:** the `TCP_NODELAY` home card and headline statistic are gone, replaced by a photography pipeline card and the connection limit. It and the Playwright benchmark mistake now share one short Corrections list at the end of the Performance page, and About Projects no longer cites the mistake as a virtue. All eight locales changed together.
- **Custom accent:** an RGB value or the native picker, alongside the unchanged default and four presets. See the [design system](../design/fe/design-system.md). Six extra presets were tried first and dropped at the owner's request; they were never committed.
- **Initial bundle limit lifted:** the 130 KiB build failure is gone at the owner's request. The renamed `build/initialAssetReport.ts` still reports the size in each build. The About pages advertised the limit as enforced, so the statistic and the loading section changed in all eight locales.

Found while doing that, or by an independent review of the first pass:

- **Chat:** bursts stopped auto-scrolling, because each message measured the distance before the previous one rendered. A full compact window stopped following at all, because the effect watched the list's length. Signing out left the socket on the old identity. Message times ignored the site language.
- **Home:** a failed posts or photographs request replaced the whole page with the application error screen.
- **Forum:** quadratic regexes on user text could freeze a tab on a crafted body. A failed search left the last query's rows and cursor in place, so "load more" could splice two queries. Every deleted account's replies could be marked as the topic author, since they share one placeholder id. A hidden topic's breadcrumb read "Loading" forever.
- **Contour field:** switching tabs during the speed measurement could freeze it until reload, and the animation switch could not revive it. Cleanup passed an idle-callback id to `clearTimeout`.
- **Appearance menu:** tabbing past the end left the panel open over the page.
- **Footer clearance:** desktop pages reserved a fixed 3rem while the footer measures about 58px at 1440 and 86px at 1024 wide, leaving the end of every page partly behind it. `.site-main` now uses the measured height, as phones already did.
- **Stale public copy:** the About Blog pages claimed the operating system's reduced-motion setting was honored, that accents were defined in OKLCH, and about 650 interface keys. They now match the code.

Decisions:

- Keeping the greeting in the provider avoids both a request per route change and reconnecting, which would drop a call in progress on the shared socket.
- A custom color is used exactly when it clears 4.5:1 and is otherwise moved only as far as each mode needs, with a note saying so. The accent colors links and labels, so exact pure yellow on cream would be unreadable.
- Custom accent code is a lazy chunk. It was split out when the follow-ups reached 130.0 KiB of the former limit, and stays split because few visitors use it.

Known limit: the server does not close a session's chat sockets on logout. The tab that signs out reconnects, but another open tab stays on the old identity until it reloads or the socket drops.

## Verification

Follow-ups, on the final working tree:

- `cargo xtask frontend-check` passed: 399 Vitest tests, 17 optimization tests, and an initial route graph of 127.6 KiB gzip.
- `npx playwright test --project=chromium` from `solid-csr-spa-template/`: 308 passed, 1 skipped.
- `npm run test:e2e:security -- --project=chromium`: 38 passed, with the new pre-paint script hash in `e2e/security-headers.json`.
- `cargo xtask clippy`, `fmt`, `unit`, and `openapi` passed.
- `cargo xtask secret-scan` fails, on findings that predate this work: 8 in the current tree and 12 in history, all the `generic-api-key` rule in test and fixture files, from September commits. None touches this change. The values are redacted and were not assessed, and no allowlist was added.

The new chat, home failure, and forum regression tests were each seen failing against the old code before the fixes applied. Pages were reviewed by eye at 390, 1024, 1440, and 1920 pixels against mocked APIs. The contour field's tab-switch fix has no dedicated test. Not run: WebKit projects, `test:e2e:performance`, and any optimized build of the follow-ups.

First pass, at `5d63813`:

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
