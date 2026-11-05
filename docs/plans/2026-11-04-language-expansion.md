# Language expansion: sixteen languages, right-to-left, stable control sizes

- Status: complete, 2026-11-04. Added the most-spoken languages the site lacked, made right-to-left correct, and stopped controls changing size when the language changes.
- Checkout: `main`; see `git log --oneline origin/main..main` for anything unpushed.
- Completed: see [Scope](#scope) and [Decisions](#decisions).
- Verification: see [Verification](#verification).
- Remaining: native-speaker review of the new languages (see [Open items](#open-items)), then install the optimized build at `miniserver:~/` through the normal deployment workflow (see [Optimized build](#optimized-build)).

## Scope

- **Languages:** Arabic (`ar`), Hindi (`hi-IN`), Bengali (`bn-BD`), Telugu (`te-IN`), Portuguese (`pt-BR`), Indonesian (`id-ID`), Russian (`ru-RU`), Urdu (`ur-PK`), Vietnamese (`vi-VN`), Tagalog (`tl-PH`), Persian (`fa-IR`), Italian (`it-IT`), Polish (`pl-PL`), Swedish (`sv-SE`), Serbo-Croatian (`sh`), and Romanian (`ro-RO`), for 24 in total. Each has a full interface catalog and both About sections.
- **Right-to-left:** the document direction follows the language, layout moved to logical CSS, and left-to-right islands and `dir="auto"` user content were applied across the site. See [localization](../design/fe/localization.md).
- **Stable sizes:** the top bar has equal navigation cells and fixed controls, `.ui-button` is a fixed box, and tabs, tables, and navigation lists no longer take their size from their labels. See [control sizing](../design/fe/design-system.md).
- **Name renditions:** after the Chinese renditions, the home hero and About eyebrow show the name in Arabic, Urdu, Persian, Hindi, Bengali, Telugu, and Russian script. Latin-script languages needed none.
- **Live chat times:** each message now shows its full date, not only the time.
- **Phone titles:** routes outside the navigation showed an English title derived from the URL in every language. They now use translated page titles.

## Decisions

- "Iranian" was taken as Persian (Farsi). Serbo-Croatian is one Latin-script catalog under the `sh` tag, which `sr`, `hr`, `bs`, and `cnr` browsers resolve to; its reference row borrows Serbian's language key and Serbia's country key, since `iso_language` has no `sh` row.
- About modules were translated from flat string maps and regenerated, rather than edited as code, so a translator could not damage ids, links, or structure. Figures carry ASCII digits in every language so the count-up numbers keep working, and Latin-script languages use their own separators as the German and Spanish pages already did.
- No web fonts were added. Arabic, Devanagari, Bengali, and Telugu use platform faces, as Korean and Chinese body text already did, so the bundle is unchanged; the cost is that the look varies by device.
- Urdu uses a Naskh-style face rather than Nastaliq. Nastaliq needs far more line height than the fixed control sizes allow, and it is rare in interface text.
- A control may cut its label only if a `title` holds the full text. The site footer's status lines are the exception: they are diagnostics in a fixed strip, and hovering reveals the full text.
- No new interface keys were added, so the backend registry is unchanged.

## Corrections

The first translation pass's checker treated numeric table cells as untranslated, and some revisers changed digits to satisfy it. The rule was corrected and the digits restored. That revision pass had also sampled rather than read everything, so a second pass compared every string with its source and made 659 corrections by its own count.

A Serbo-Croatian term replacement in the first revision pass ran Perl without `-Mutf8`, which double-encoded `č`, `ć`, and `đ` in the words it inserted. The second pass caught it. A scan of every catalog and About module afterward found no corrupt sequences.

A malformed line in the first committed version of `e2e/layout-stability.spec.ts` stopped it loading for a time, and an edit that truncated `TopBar.tsx` before rewriting it let the dev server cache an empty module, which blanked every page. Results from those periods were rerun rather than trusted.

A physical `margin-left` on `.threaded-comment` in `src/index.css` stacked with the logical indent in right-to-left, squeezing deep comment threads to 4 pixels. Only widening the sweep to the photograph lightbox exposed it.

The router scrolls to an administration section before the panels above it finish loading, so the target drifted back down. This race predates the work, and taller controls made it visible. `AdminWorkspace` now follows the section until the panels settle or the reader scrolls.

## Dependencies

Frontend packages moved to their newest compatible releases: Solid and web `2.0.0-rc.21`, router `2.0.0-next.44`, Vite plugin `3.0.0-next.56`, `oxlint`, `turndown`, `@types/node` 26, and transitive updates. Solid packages stay on their `next` tag because `latest` points at older lines. The Docker frontend build moved from Node 24 to Node 26.13, the newest stable release, because nothing required 24 and local work already ran on 26. `Cargo.lock` carries the owner's `cargo upgrade`.

The initial route graph is 131.5 KiB gzip, up from 127.6 KiB, from the locale list and direction helpers. The size limit was lifted earlier, so this is reported only.

## Open items

- **Native review:** no native speaker has read the new languages. Revisers specifically doubted Serbo-Croatian words that lean Serbian or Bosnian (`sajt`, `računar`, `univerzitet`), Tagalog terse labels and `Mga teknolohiya` for Stack, Arabic `وزير دولة` for state secretary, and Russian `Неразрешённая очистка медиа`.
- **Still English in every language, deliberately left:** the Minecraft map interface, the repository source map on About Blog, the Markdown editor's toolbar, and the map address search placeholder. These are independent applications or third-party surfaces, and the owner chose to leave them. Browser validation bubbles and file-picker labels follow the browser's language.
- **Plain text in user content:** an Arabic sentence with an inline unit such as `2.38 ms` reorders under standard bidi rules, and a sentence opening with a Latin word lays out left to right outside live chat.

## Verification

On the final code:

- `cargo xtask frontend-check` passed: 422 Vitest tests, 17 optimization tests, and the 131.5 KiB gzip initial graph.
- `npx playwright test --project=chromium` from `solid-csr-spa-template/`: 327 passed, 1 skipped.
- `npm run test:e2e:layout`: 165 passed with no offender lines, covering 40 route and sign-in combinations at 390, 1024, 1440, and 1920 pixels plus five overlays, each in all 24 languages. `/minecraft` is left out because the owner chose to leave that application alone.
- `npm run test:e2e:security -- --project=chromium`: 38 passed.
- `cargo xtask clippy`, `fmt`, `unit`, and `openapi` passed with the upgraded `Cargo.lock`, and `cargo test --locked --package xtask` passed 82.
- `cargo xtask secret-scan` passed.
- `docker build --target frontend-source` passed on the new image, and `npm run build` inside it passed offline on Node 26.13.
- `npm audit` reports no vulnerabilities and `npm ls --all` no problems.

Size differences across languages fell from 12,055 on the first sweep of 96 route and width combinations, in the original eight languages, to none across 24 languages. The right-to-left review opened 834 screenshots in Arabic, Urdu, and Persian with mixed-direction content, by its own count, though two of its second-look passes relied on code reading alone.

Not run: WebKit, which cannot launch on this host because `libicudata.so.74` is missing, so right-to-left is unverified in Safari; `test:e2e:performance`; and `cargo xtask db-integration`, whose readback of each language's rows needs a disposable database. The optimized build covers part of the last one: every backend start synchronized 17,712 source rows, 738 for each of the 24 languages, into a real PostgreSQL.

## Optimized build

`./build_pgo_and_bolt.sh` from `e66f2fb` was accepted as run `cyhdev-pgo-bolt-18e50257deb45a6a_b989a`, with its frontend compiled on the Node 26 image. Browser coverage passed on all five stages. BOLT against baseline measured 1.199x throughput, 0.834x p95, and 0.836x p99, the median of three paired rounds. Absolute throughput fell across the rounds, from 53,241 to 45,302 requests/s for the baseline, and the per-round BOLT throughput ratio ranged from 1.195x to 1.312x, so these ratios are approximate. The home page shows `+20.1% req/s`; this build measured 19.9%, and the figure was left unchanged.

`rust-be-template` (`4b0af43e…e769`) and `minecraft-seed` (`1481d16b…8b9f`) are at `miniserver:~/`, and their SHA-256 hashes match the receipt on both ends. Neither is installed, and the service was not touched. `~/dist` holds the `a140267` binaries.
