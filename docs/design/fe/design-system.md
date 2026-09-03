# Frontend design system

Applies to `solid-csr-spa-template/`. Source of truth for tokens is `src/index.css`; composed component classes live in `src/styles/pageStyles.ts` (the single style source for Tailwind semantic strings); prose and code-block styling in `src/styles/prose.css` and `src/styles/code.css`.

## Direction

Technical-brutalist, warm. Cream paper in light mode, pure black in dark mode, warm neutrals in both, one amber accent. Monospace for anything technical (nav, labels, metadata, status bar); sans for reading copy. Hard 1px borders carry structure; shadows are avoided except for floating chrome (menus, drawers). No scroll animation, no page transitions; motion is confined to hover/focus/theme transitions under 300ms.

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

`--glow-ok` / `--glow-danger` are 60% color-mix halos for the status dots (`shadow-[0_0_8px_2px_var(--glow-ok)]`).

Policy: amber is the only decorative accent. `ok`/`danger` are semantic only. No raw `slate-*`/`gray-*`/`blue-*` palette classes; the audit `grep -rnE "(slate|gray|blue|amber|emerald|rose|red)-[0-9]" src/` must stay empty. Inline `<style>` blocks (projects, photographs, visitor board) use `var(--...)` directly; literal colors are allowed only for mode-independent areas: image letterboxes, overlay scrims, on-image controls.

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
- Motion: `transition-colors duration-90` for chrome, up to 200-300ms for icon swaps; everything gated by `prefers-reduced-motion: reduce`.
- Grain: a fixed `body::before` SVG-noise overlay (2.5% light / 4% dark) above the ground, below content.

## Code blocks

`src/styles/code.css` is a hand-written highlight.js theme on the dark surface (`#141210`) used in BOTH modes; dark code blocks on cream are intentional. Prose maps `--tw-prose-pre-bg` to the same surface. Inline code renders as a bordered `surface-2` chip without backtick pseudo-content.

## Repository source links

The tech-stack page builds implementation links from `VITE_REPOSITORY_SOURCE_BASE_URL` and typed monorepo-relative paths in `src/config/sourceLinks.ts`. The base must point to the repository file root for a branch or commit, such as `https://github.com/younghyun1/cyhdev/blob/main`; production can replace `main` with a commit hash when permanent historical links are required. Keep provider and revision configuration in the base, keep component paths relative to the monorepo root, and do not add line fragments. Source-map layout classes remain centralized in `src/styles/pageStyles.ts`.
