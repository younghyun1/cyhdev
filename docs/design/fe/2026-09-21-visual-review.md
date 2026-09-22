# Page renders and visual review

The route inventory in [the site map](site-map.md) drives `solid-csr-spa-template/e2e/page-renders.spec.ts`: 33 route states, 1440×900 and 390×844 viewports, light and dark themes, producing 132 full-page PNGs under ignored `target/page-renders/`. Reproduce from the root with `npm --prefix solid-csr-spa-template run test:e2e:chromium -- page-renders.spec.ts --workers=4`. Each capture checks the exact document title, horizontal overflow, and uncaught browser errors. Local `target/page-renders/index.html` and four contact sheets provide the review gallery; these generated artifacts are not committed.

## Corrections

- Contain Leaflet layers in the visitor map's stacking context so the mobile drawer and page chrome remain above them. Keep attribution below the map. The browser regression checks the actual hit-tested element in the overlapping area, then follows a drawer link.
- Distinguish missing routes from the intentionally unfinished page. `/404` and unmatched routes now say that the page does not exist; only `/under-construction` shows construction status.
- Keep profile images from shrinking into ovals beside long names. Keep the missing-page icon square on narrow screens.
- Use `Young Hyun Chi | Software Engineer` for the document title and both existing locales' desktop title, including server-backed translation sources.
- Add confirmed superuser message deletion with visible failure feedback. Both the full chat page and shared compact panel use the same control.

## Coverage limits

These are deterministic fixture screenshots, not production captures. Authentication and content APIs are mocked; photographs use a placeholder image, map tiles are blocked, and the Minecraft map and EU5 iframe show explicit local-preview unavailability. The project list uses fixture metadata instead of running uploaded WASM. Swagger, actual embedded applications, media delivery, and live RTC calls need their respective running services for visual verification. These external surfaces are inventoried in the site map but are not represented as successfully rendered application content.

The review inspected the four contact sheets and the affected map, missing-page, and profile layouts. Extremely long About, profile, forum, and article pages need full-size inspection for content-specific issues that thumbnails cannot establish. Existing Solid development warnings about untracked reads in profile-picture and effect callbacks remain follow-up work; they are not counted as uncaught page errors.

## Suggested polish

1. Bring host-stat panels and charts onto the shared cream/black/amber tokens. Their blue-gray gradients and heavy shadows are visibly inconsistent with the rest of the site. Preserve distinct chart series labels and contrast.
2. Add a featured-project case-study panel to the home page with an architecture thumbnail, concise outcome, and link to a working demo. Use repository-native SVG or HTML, not a new image dependency.
3. Give the photography preview a curated cover and compact location/date captions. Verify with real media before choosing crops; white fixture images are not an application defect.
4. Add a compact section index to long About and technical-article pages. Prefer normal anchor navigation, visible keyboard focus, and reduced-motion support over scroll animations.
5. Add explicit branded loading/unavailable states around embedded applications, preserving their own navigation and avoiding a permanent spinner when an external service fails.

These are proposals, not claims of implemented redesign. Keep the existing restrained visual system, lazy route assets, and mobile performance budgets. Translation into French, Spanish, Simplified Chinese, Traditional Chinese, Japanese, and German is the next separate scope; update frontend defaults, backend source bundles, locale selection, and layout coverage together.
