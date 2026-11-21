# KCNA archive section

The archive is a section of the website for superusers: a search-first research tool over about 400,000 articles of the Korean Central News Agency in six languages. It is reached from the Projects page and the admin workspace navigation, shares the site's shell, tokens, typography, and components, and talks to the backend only through the generated client. The backend side is described in [archive serving](../../architecture/be/kcna-archive-serving.md) and the [backend feature notes](../../architecture/be/kcna-archive-feature.md).

## Routes and access

| Route | Page |
| --- | --- |
| `/projects/kcna-archive` | Search: query form, filters, month histogram, results, and the reader beside them |
| `/projects/kcna-archive/documents/:documentId` | One article; the citation link, and the reader on narrow screens |
| `/projects/kcna-archive/trends` | Up to four terms compared over time |
| `/projects/kcna-archive/status` | Snapshot summary and the reload action |

Each route is lazy-loaded and wrapped in `withAuth(withSuperuser(...))` in `src/routes.ts`, so a signed-out visitor goes to login and any other account goes to the site 404. Each page renders the shared section layout (`pages/kcna_archive/Layout.tsx`: title, the administrators-only badge, and the Search, Trends, and Status tabs) around its own content. The reader counts as part of Search for the active tab. Any other path below `/projects/kcna-archive` falls to the site 404. The guards control rendering only; every endpoint answers superusers only.

The entry points are rendered only when `canViewArchive()` holds (signed in and superuser, the same condition as the admin navigation): a card at the top of the Projects page with the `kcna.admin_only` badge, and a link in the admin workspace navigation. The public navigation and the command palette list public pages only, so a visitor who is not a superuser finds no trace of the section in the menus, the palette, or the Projects page.

## Code layout

| Area | Location |
| --- | --- |
| Pages | `src/pages/kcna_archive/` |
| Components | `src/components/kcna_archive/` |
| Pure logic and tests | `src/utils/kcna_archive/` |
| Service wrapper over the generated client | `src/services/contracts/kcna_archive.ts` |
| Styles | `src/styles/kcna-archive.css` (frame, controls, filters, notices), `kcna-archive-reader.css` (results and reader), `kcna-archive-charts.css` (histogram, trends, status), `kcna-archive-entry.css` (the Projects card and badge) |

Styles are imported by the layout and the entry card, so the Projects page loads only the entry stylesheet. All text comes from the site catalog (`kcna.*` keys, plus `common.*` for loading and unavailable values), and numbers and dates go through `Intl` with the selected interface language (`utils/kcna_archive/format.ts`). Dates are read as midnight UTC so the reader's time zone never moves a day.

## One query, kept in the URL

A query is text, languages, sources, a category, a date range, and a sort order. Empty text is browsing. Results are newest first unless another order is chosen, because an archive of news is read by date. `utils/kcna_archive/query.ts` converts between the URL (`q`, `lang`, `source`, `category`, `from`, `to`, `sort`) and the typed query, ignoring invalid values, and the URL is the only copy: pages read the router's search parameters and write changes back. Every view is a link, and the back button undoes the last change. Equal selections give equal URLs. The open document and the side-by-side language are the `doc` and `with` parameters on the search page; the trends page keeps `terms`, `lang`, `bucket`, and `mode` the same way.

Dragging across the histogram selects whole months. The chart always shows the whole time axis with the span marked, so a range can be widened from the same place, and clicking the single selected month again clears it. The histogram is an image for assistive technology, so the same range is available through the From and To date fields, the clear-range button, and the small Go to date form (`2026`, `2026-11`, or `2026-11-21`).

## Reading

Result rows show the date, language, every source that holds the article, the title, and a snippet with matches marked. The reader shows the article, a link to it at its source (`rel="noreferrer noopener"`), a copyable citation, the other language editions, the other sources that hold the article, and the record facts with the source metadata collapsed. There is no link to a stored copy of the page: the backend keeps none.

Any other edition opens beside the current one with paragraphs paired by position. Editions break the opening differently, so when one side has exactly one paragraph more and starts with a short dateline, that line is joined to the paragraph after it (`parallel.ts`). The two columns stack when the article is narrow, measured on the article with a container query, not on the window.

Archive text is Korean, English, Chinese, Russian, Spanish, or Japanese whatever the interface language is. Titles, snippets, and the reader body carry `lang` from the document language and `dir="ltr"`, so browsers pick regional glyphs and the site's font stacks apply. Matches are `mark` elements with a translucent accent wash and an accent underline, which read in both themes and with any accent. Article content arrives as text segments and is always rendered as text, never as HTML.

On screens wider than 1023 pixels the reader sits beside the results and stays in view while a long list scrolls. Below that width the reader is hidden, results link to the document page, and nothing is fetched for the hidden reader.

## States and requests

Every region reads an async memo under its own `Loading` boundary with a skeleton fallback, and an `Errored` boundary shows the failure where the data would appear, with a retry. The filters appear at once and gain their counts when the first page arrives; the histogram and the reader fail independently of the results. A progress line under the search field runs while a request is pending; the previous results stay on screen until the new ones arrive.

A response for a query the reader has already left is never shown: the async memos drop superseded reads, and the pages loaded as the reader scrolls (`sequence.ts`) go through a request gate that makes every outstanding page stale when the query changes. Results load 50 at a time through an `IntersectionObserver` on a sentinel below the list, with the server's 10,000 result window as the end; after a failed page the loaded results stay and loading resumes only through the retry button, so an outage cannot cause a request loop. The client keeps no cache beyond the loaded pages of the current query, and formatter caches hold one entry per interface language.

The backend answers 503 with a distinct error code when the snapshot is not installed, still loading, or failed to load (`failure.ts` maps 113, 114, and 115). Every page shows that as a plain notice with a retry, not as an error, and the status page shows the same states from its own 200 answer. Other failures read as not found, a rejected request, an unavailable archive, or an unreachable server. Article not found (112) is its own message.

The status page shows the snapshot counts, coverage, retrieval and build times in UTC, the memory held, and bars by language and source. Its Reload snapshot button posts to the reload endpoint and reports success, a reload already running (409), or a failed reload that leaves the previous snapshot serving. Export is a same-origin download link to the export endpoint carrying the current query, in CSV or JSON Lines. The trends page downloads its own table as CSV from the data already loaded.

## Keyboard

| Key | Action |
| --- | --- |
| `/` | Focus the page's main text field and select its text |
| `j` and `k` | Open the next or previous result beside the list (wide screens) |
| `t` | Toggle the side-by-side translation, English first |

The shortcuts never act while a text field, select, or editable element has focus, with Ctrl, Command, or Alt held, or while a modal dialog is open, so they leave the site's own Ctrl+K or Command+K palette alone. The decision is a pure function (`shortcuts.ts`) with its own tests.

## Visual rules

The section uses the site's tokens and furniture without additions: surfaces, 1px hairlines, `rounded-sm`-sized corners, monospace labels, square-marker section headings, the accent rule under the title, and the blueprint grid behind the page. Records are hairline rows inside one bordered box. Light and dark mode come from the theme state and the accent from the accent state; no color is declared in the section's stylesheets. The trends chart reads the same tokens, and its series differ by dash pattern as well as color. Charts and the histogram stay left to right in every language, like the statistics charts, and everything else uses logical properties so right-to-left languages mirror.

Controls take their size from the stylesheet, never from their label: buttons are fixed boxes whose label wraps inside, filter toggles and tabs are equal cells with a title that holds a cut label, and the trends table is fixed-layout with titled headers. Below 768 pixels inputs and buttons keep the 44 pixel target, filter toggles let a label take two lines, and tables and the status columns reflow without the page scrolling sideways. Animations are limited to a skeleton pulse, the load line, and the Projects card's accent sweep, and all of them stop when the site's animation switch is off.

## What the standalone app had that this section does not

The standalone browser had its own design tokens, a bundled variable font, a theme toggle, a language toggle, a toolbar, and a command palette. The section uses the site's equivalents instead. Its other differences:

- The date-jump command of the palette is the Go to date form near the date filters; the other palette commands are covered by visible controls.
- View transitions between a result and the reader, the sliding selection marker, staggered rows, and animated counters are gone, because the site has no equivalent pattern and its animation switch governs motion.
- Runtime decoding of responses is gone: types come from the generated client, and the wrapper adds only the repeated-parameter and error handling the site's transport provides.
- The stored raw page link is gone with the raw page endpoint.
- Its own language and theme preferences and its `/d/:id` and `/trends` routes became the site's state and the routes above.

## Verification

`npm --prefix solid-csr-spa-template run test` covers query parsing, histogram geometry, trends, paragraph pairing, request sequencing, shortcuts, formatting, failure mapping, the service wrapper's query strings, the superuser-only visibility of the Projects entry, and the routes. `npm --prefix solid-csr-spa-template run test:e2e:chromium -- kcna-archive.spec.ts` exercises access, search, shortcuts, the reader, states, trends, and status against mocked endpoints (`e2e/kcna-fixtures.ts`, shared with the page-render and layout specs). Running it with `KCNA_RENDERS=1` also writes screenshots of the section in English, Korean, Arabic, and German, light and dark, at 390 and 1440 pixels to `target/kcna-archive-renders/`. `npm --prefix solid-csr-spa-template run test:e2e:layout` includes the archive routes in every language.
