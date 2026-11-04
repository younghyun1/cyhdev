# Interface pass: home board, forum, animation switch, page styling

- Status: complete, 2026-11-04. All nine interface items are implemented and the optimized build reached `miniserver:~/`; nothing is pushed or deployed.
- Checkout: `main`; see `git log --oneline origin/main..main`.
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
- The panel height on desktop leaves room for the hero, so the composer is visible on arrival. Viewports under 560px tall drop the sticky position.
- 29 UI keys were added to all eight catalogs, the browser key lists, and the backend registry; `forum.topic.back` and `forum.topic.created` became unused and were removed.
- The inline pre-paint script changed, so `e2e/security-headers.json` carries a new script hash. The backend computes its own hash from the built shell at startup.
- The optimization campaign's route assertions in `tools/optimization-fixtures/src/campaign.rs` follow the renamed topic heading and profile name.

## Corrections

The contour field's slow-device probe averaged the interval between drawn frames and stopped animating above 28ms. Frames are throttled to at least 33ms apart, so the average always exceeded the limit and the field froze after 20 frames, about 0.7 seconds, on every machine. The November 2 verification only confirmed that it started. The probe now sees every animation-frame callback and stops only below 20 per second; `contourField.test.ts` covers it.

The `/about` portrait stretched into a wide pill on phones, and the `/about-blog` tabs overlapped at 390 pixels. Both predate this work and are fixed.

