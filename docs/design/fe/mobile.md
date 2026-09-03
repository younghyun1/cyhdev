# Solid mobile conventions

Applies to every route in `solid-csr-spa-template/`. Mobile behavior ends at 767 CSS pixels; 768 pixels and wider is the protected desktop baseline. New visual overrides belong in CSS under `@media (max-width: 767px)`. JavaScript branches use the typed `createMediaQuery(query): Accessor<boolean>` wrapper only when behavior cannot be expressed in CSS.

## Viewport and fixed chrome

The viewport meta tag uses `viewport-fit=cover`. `--safe-area-top`, `--safe-area-right`, `--safe-area-bottom`, and `--safe-area-left` wrap the corresponding environment insets. The application measures the rendered top and bottom bars with `ResizeObserver` and publishes `--site-header-height` and `--site-footer-height`; viewport-bound routes must use these variables with `dvh` instead of copying bar heights. The compact status bar remains a 44px mobile target, includes the bottom safe area, and hides while a text-entry control has focus so the browser keyboard does not cover the active form. The EU5 wrapper keeps eager iframe loading and its measured bar offsets. These changes optimize only the Solid-owned wrapper; the embedded Slint layout remains outside the mobile-responsiveness contract.

## Interaction and accessibility

Mobile text inputs, selects, and textareas render at 16px with a minimum 44px block size. Primary buttons and navigation actions use a 44px target; smaller custom controls retain at least 24px. Global focus-visible styling remains mandatory, and reduced-motion preference removes meaningful transition duration. The shared portal-backed dialog lifecycle locks body scrolling, marks the application root inert, traps Tab and Shift+Tab, closes on Escape or backdrop activation, and restores focus to the opener. Use it for drawers, bottom sheets, confirmations, and full-viewport mobile media or upload surfaces. Long text must use wrapping or a contained horizontal scroller; the document itself must never scroll horizontally at 320 CSS pixels.

## Route patterns

- Content and account pages use the semantic hooks in `pageStyles.ts` for mobile gutters, heading scale, form sizing, and card padding. Long forms are top-aligned so browser scrolling can keep focused controls visible.
- Blog vote controls become a horizontal rail. Prose tables and code blocks own their horizontal scrolling. Threaded blog and photograph comments use a bounded incremental indent that stops after three mobile levels.
- Full live chat uses the dynamic viewport remaining between the measured site bars. The composer remains inside that surface; call tiles and controls wrap without introducing document overflow.
- Photograph cards reserve a cropped 4:3 mobile image area while detail sheets show the uncropped original. Mobile list requests use 12 records; desktop requests use 24. Detail maps, geosearch/upload code, and batch processing are lazy chunks, with eager module prefetch below the route boundary on desktop. Detail images support horizontal touch swipes.
- Project grids collapse to a zero-minimum single column. Solid-owned demo and upload dialogs fill the mobile visual viewport, and hosted demos expose an open-separately action because their internal layout is not controlled here. Visitor and photograph maps observe their containers and invalidate after resizing.
- Statistics reduce panel and chart height, hide redundant legends, and limit tick density on mobile. The statistics WebSocket closes while a mobile document is hidden and reconnects when visibility returns; desktop streaming remains continuous.
- Administration keeps the compact workspace strip. Authorization and retention tables render as labeled cards below 768px, searches and record actions stack, and confirmation dialogs become bounded keyboard-safe bottom sheets.

## Delivery and verification

Route-level lazy loading remains the default. Below-fold About media uses lazy loading and asynchronous decoding only on mobile; desktop keeps eager loading. The Vite `initial-asset-budget` plugin follows static imports from each entry chunk, includes their stylesheet assets, excludes dynamic route chunks and non-code assets, and fails above 130 KiB gzip. `npm run test:e2e` runs the deterministic Chromium and WebKit layout, interaction, and accessibility suite. `npm run test:e2e:performance` builds the production frontend and records five cold-cache Chromium runs at 390×844 with Fast 4G and 4× CPU throttling; median targets are LCP at most 2.5 seconds, interaction duration at most 200ms, and CLS at most 0.1 for home, blog, forum, photographs, login, and live chat.
