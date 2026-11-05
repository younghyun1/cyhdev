# Frontend design system

Applies to `solid-csr-spa-template/`. Source of truth for tokens is `src/index.css`; composed component classes live in `src/styles/pageStyles.ts` (the single style source for Tailwind semantic strings); prose and code-block styling in `src/styles/prose.css` and `src/styles/code.css`.

## Direction

Technical-brutalist, warm. Cream paper in light mode, pure black in dark mode, warm neutrals in both, one decorative accent at a time (amber by default). Monospace for anything technical (nav, labels, metadata, status bar); sans for reading copy. Hard 1px borders carry structure, with faint blueprint grids and accent washes behind heroes; shadows are reserved for floating chrome (glass header, menus, drawers, palette). Motion is deliberate and cheap: see [Motion](#motion).

## Control sizing

Changing the language must not move anything. A control's size never comes from its label: the top bar uses equal navigation cells in `src/styles/header.css`, `.ui-button` is a fixed box in `src/index.css` whose label wraps inside it, tab strips use equal cells, and tables use `table-layout: fixed`. A label may be cut only with `.ui-clip` and a `title` holding the full text. Running text may reflow; controls may not. `npm run test:e2e:layout` loads each route and overlay in every language and fails if a control's size differs from English, a label is cut without a title, or a bar moves. It takes several minutes, so it is separate from the main browser suite.

## Tokens

Semantic CSS variables declared on `:root` (light) and `.dark`, exposed to Tailwind through `@theme inline` so utilities are mode-aware without `dark:` variants.

| Token | Utility | Light | Dark | Role |
| --- | --- | --- | --- | --- |
| `--paper` | `bg-paper` | `#f6f1e8` | `#000000` | page ground |
| `--surface` | `bg-surface` | `#fdfaf4` | `#141210` | cards, panels, inputs |
| `--surface-2` | `bg-surface-2` | `#f1eadd` | `#1d1a17` | wells, hover fills, skeletons |
| `--ink` | `text-ink` | `#211d18` | `#f0ece5` | primary text |
| `--ink-muted` | `text-ink-muted` | `#6e655a` | `#a39a8f` | secondary text |
| `--ink-faint` | `text-ink-faint` | `#9a9084` | `#6f675d` | tertiary text, separators |
| `--line` | `border-line` | `#e3dacb` | `#262220` | hairline borders |
| `--line-strong` | `border-line-strong` | `#c9bfae` | `#3d3731` | emphasized borders |
| `--accent` | `text-accent` etc. | `#b45309` | `#fbbf24` | links, active states, markers |
| `--accent-strong` | | `#92400e` | `#fcd34d` | accent hover |
| `--accent-soft` | `bg-accent-soft` | `#f6e5c4` | `#422006` | accent-tinted chips, selection |
| `--ok` | `text-ok`/`bg-ok` | `#047857` | `#34d399` | success, upvotes, online |
| `--danger` | `text-danger`/`bg-danger` | `#be123c` | `#fb7185` | errors, downvotes, delete |

Derived tokens follow whichever accent is active: `--accent-glow` (45% mix, halos and seams), `--accent-wash` (12% light / 16% dark, hero washes and card spotlights), `--grid-line` (6-7% ink, blueprint grids), and `--glass` / `--glass-highlight` (translucent header ground and its top specular line).

Accent presets: `data-accent` on `<html>` selects `lagoon`, `cobalt`, `orchid`, or `graphite`; no attribute means amber. Each preset overrides only `--accent`, `--accent-strong`, and `--accent-soft` per mode in `index.css`. Presets keep at least 4.5:1 contrast on `--paper`, `--surface`, and `--accent-soft` in each mode (amber keeps its historical values); `e2e/site-interface.spec.ts` checks this in the browser. The pre-paint script in `index.html` applies the stored preset with the theme; `src/state/accent.ts` owns the choice afterwards. The header appearance menu, the command palette, and the mobile drawer all expose it.

Custom accent: `data-accent="custom"` is the visitor's own color, entered as an RGB triplet or hex value or through the native picker in the appearance menu and the drawer, not the command palette. `src/utils/customAccent.ts`, in a chunk loaded only when a visitor uses this, derives `--accent`, `--accent-strong`, and `--accent-soft` per mode, using the color exactly when it clears the same 4.5:1 floor and otherwise moving it toward black or white only as far as that mode needs; the field says so when it did. The derived values are stored as ready-made declarations in `accent-custom`, so the pre-paint script applies them without repeating the math.

Policy: one decorative accent at a time. `ok`/`danger` are semantic only, so no preset uses green or rose; a visitor's own color may. No raw `slate-*`/`gray-*`/`blue-*` palette classes; the audit `grep -rnE "(slate|gray|blue|amber|emerald|rose|red)-[0-9]" src/` must stay empty. Route stylesheets and inline `<style>` blocks use `var(--...)` directly; literal colors are allowed only for mode-independent areas: image letterboxes, overlay scrims, on-image controls.

## Typography

- Sans: IBM Plex Sans 400/400i/500/600/700; body and reading copy. `IBM Plex Sans KR` (700) covers the hangul/hanja hero glyphs via unicode-range slices.
- Mono: IBM Plex Mono 400/500/700; headings on home, nav, tags, metadata lines, footer status bar.
- Self-hosted via `@fontsource/*` imports in `index.css`; woff2, `font-display: swap`, subset by unicode-range. No weight above 700 exists in Plex: use `font-bold`, never `font-black`.
- Numerals that update in place (uptime, latency, dates, scores, pagination) get `tabular-nums`.
- Prose measure is capped at 70ch (`.prose` in prose.css).

## Radii

`rounded-sm` for cards, panels, buttons, inputs, menus. `rounded-full` only for pills, avatars, status dots. No `rounded-md/lg/xl`.

## Dark/light mode

- Class strategy: `.dark` on `<html>`; state in `src/state/theme.ts` (localStorage `theme` -> `prefers-color-scheme` -> light).
- A blocking inline script in `index.html` applies the class and `meta[name=theme-color]` before first paint (no flash); `applyTheme()` keeps the meta in sync on toggle.
- `color-scheme` is set per mode so native controls and scrollbars match.
- The EU5 iframe receives the resolved mode through the bounded same-origin `cyhdev:eu5-theme:{light|dark}` protocol. It announces readiness before WASM initialization so the initial mode is queued without reloading the iframe, then interpolates its semantic palette over the same 90ms default easing as the Solid shell.
- Components should not need `dark:` variants; if one seems required, the token set is missing a role.

## Interaction

- Focus: global `:focus-visible { outline: 2px solid var(--accent) }`; never `focus:outline-none` without replacement.
- Links: amber, `underline-offset-4`, decoration fades in via `decoration-accent/40 -> decoration-accent`.
- Nav: active route gets ink text + 2px amber underline (desktop) or amber left border + `bg-surface-2` (drawer).
- Selection: `::selection` uses `--accent-soft`.
- Grain: a fixed `body::before` SVG-noise overlay (2.5% light / 4% dark) above the ground, below content. The body ground also carries a non-fixed accent wash at the top of the first screen.
- Header: `.site-header-glass` in `styles/header.css` is translucent paper with an 18px backdrop blur, a top specular highlight, and an accent seam; `prefers-reduced-transparency: reduce` restores opaque paper.
- Appearance menu: the header button shows the active accent and opens a popover with the accent swatches and the animation switch; the mobile drawer carries the same two controls. Outside presses and Escape close it, and Escape returns focus to the button.
- Command palette: Ctrl+K or Command+K (ignored inside other text fields), the header search button, or the mobile drawer opens a lazily loaded combobox dialog over public pages, About subpages, theme, animation, and accent actions. It reuses the shared dialog lifecycle for focus containment and inert background.

## Motion

Motion is allowed where it explains structure or adds polish at negligible cost. Rules:

- Animate only `transform`, `opacity` on decorative layers, or registered custom properties. Never animate text opacity on content, so axe contrast checks and readers never meet half-faded text.
- Prefer CSS over script: scroll-driven animations (`animation-timeline: view()` / `scroll()`) for section reveals, benchmark bars, statistics, and the reading-progress bar; `@property` for the count-up integers and the portrait ring angle; `@supports` gates every timeline feature, and unsupported browsers see the final state.
- Chrome transitions stay at 90ms for color, up to 200-300ms for icon swaps and hover lifts. Route content enters with a short transform-only rise.
- The theme toggle uses the View Transitions API to reveal the new palette as a circle from the toggle; without the API it switches instantly.
- The home hero's WebGL2 contour field (`components/home/`) loads after idle unless Save-Data is on. Desktop-width fine-pointer devices animate it; other devices, and everyone with animations off, draw one still frame. It drifts the terrain, raises the contour levels so lines keep migrating, and runs a soft brightness swell through the bands. It caps device pixel ratio at 1.25 and frame rate at 30, pauses off-screen and in hidden tabs, and re-reads `--accent` and `--ink-faint` when the theme or accent changes. It stops on its last frame only if animation-frame callbacks average under 20 per second; the probe must see every callback, because the interval between drawn frames is at least 33ms by construction.
- The animation switch is the only control. `src/state/motion.ts` owns it, `data-motion="off"` on `<html>` gates the stylesheets, and the pre-paint script in `index.html` applies a stored choice. Animations are on by default and the operating system's `prefers-reduced-motion` is deliberately not consulted, at the site owner's request. Switched off, the global rule shortens durations, and timeline-driven, infinite, or entrance animations are set to `none` explicitly: duration does not stop the first kind, and a pending entrance would hold its first frame until the next paint. The photograph viewer keeps its existing exception: a decoded image fades in over 180ms, and a thin accent line sweeps only while a load has taken longer than 150ms.
- Entrance animations use a backwards fill only. A forwards-filled transform would leave the element a stacking context and a containing block for the fixed modals inside it.

## Page furniture

`styles/surfaces.css` gives every route the look of the About pages: a faint blueprint grid at the top of the page, titles carrying a short accent rule, section headings with a square accent marker, and an accent edge on authentication cards. Components get these through `pageStyles.title`, `titleSm`, and `sectionTitle` rather than per-page styles. Lists of records (home posts, blog list, forum topics) are hairline rows inside one bordered box with a faint accent hover, not separate cards. Nothing in this file may create a stacking context on `.app-page`, because modals inside pages rely on the root stacking order against the fixed header.

## Content pages

About Me and About Blog render typed content (`src/pages/about-locales/content_types.ts`) through `components/content/`. Blocks cover paragraphs, lists, callouts, statistics, role timelines, link cards with a pointer spotlight, a request pipeline with a travelling packet, tag groups, benchmark bars, and tables. Copy uses a small inline syntax (`**strong**`, `` `code` ``, `[label](href)`) parsed by `utils/inlineMarkup.ts` into elements, never `innerHTML`; only `https:`, `mailto:`, and same-origin paths become links. Styles live in `styles/content-page.css`; the reading-progress bar is shared with blog posts through `styles/motion.css`.

## Code blocks

`src/styles/code.css` is a hand-written highlight.js theme on the dark surface (`#141210`) used in BOTH modes; dark code blocks on cream are intentional. Prose maps `--tw-prose-pre-bg` to the same surface. Inline code renders as a bordered `surface-2` chip without backtick pseudo-content.

## Repository source links

The tech-stack page builds implementation links from `VITE_REPOSITORY_SOURCE_BASE_URL` and typed monorepo-relative paths in `src/config/sourceLinks.ts`. The base must point to the repository file root for a branch or commit, such as `https://github.com/younghyun1/cyhdev/blob/main`; production can replace `main` with a commit hash when permanent historical links are required. Keep provider and revision configuration in the base, keep component paths relative to the monorepo root, and do not add line fragments. Source-map layout classes remain centralized in `src/styles/pageStyles.ts`.
