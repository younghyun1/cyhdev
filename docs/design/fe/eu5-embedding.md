# Embedded EU5 Locations DB application

The `/eu5-locations-db` page embeds the vendored EU5 location filter, an iced application compiled to WebAssembly. This document covers what site maintainers need: how the frame is built and secured, what the page sends, and how assets are staged. Message shapes, validation rules and the WebAssembly exports are defined once, in the [host protocol](../../../vendor/eu5-location-filter/docs/design/fe/host-protocol.md) of the application repository; this document does not repeat them.

## Frame and trust

The page `solid-csr-spa-template/src/pages/eu5_locations_db.tsx` renders one iframe for `/eu5-locations-db/app/index.html`. The frame is sandboxed with `allow-scripts allow-popups allow-popups-to-escape-sandbox allow-top-navigation-by-user-activation` and without `allow-same-origin`, so it has an opaque origin and cannot read site storage or call the API with the viewer's cookies. The server sends the same sandbox as a response Content-Security-Policy with `frame-ancestors 'self'`; see [browser security headers](../../architecture/be/browser-security-headers.md). Because the origin is opaque, the page posts with target origin `"*"` and identifies the frame by its window and the `"null"` origin. Messages carry public presentation settings and a view-state string only.

## Protocol

The page and the frame speak protocol version 2 of the [host protocol](../../../vendor/eu5-location-filter/docs/design/fe/host-protocol.md). The page changes together with the vendored host script; nothing accepts the older theme-only string messages. Protocol helpers (message construction, palette resolution, validation) live next to the page in the frontend, and their unit tests cover them.

## What the page sends and when

- **On ready.** After each ready message from the current frame window, the page sends a full host message: the twelve resolved theme colours and mode, the site locale and direction, the motion setting, and the validated `view` query parameter when present. A frame reload sends a new ready and gets a new full message.
- **On change.** When the theme mode, accent, custom accent, locale or motion switch changes, the page sends the settings again without a view state. A mutation observer on the root element's `class`, `data-accent`, `style`, `lang`, `dir` and `data-motion` attributes triggers this, and the page skips a send equal to the last one.
- **Never early.** Nothing is sent before the first ready.
- **Reduced motion.** The flag follows the site's own motion switch, not the operating system setting.

## The view query parameter

The frame reports its canonical view state after the dataset is ready. The page writes it to the `view` query parameter with history replacement, so Back leaves the page instead of stepping through filters, and omits the parameter for the default view. Opening a URL with `view` makes the page pass it as the initial view state, so filtered views can be shared as links. Invalid values are ignored.

## Staged assets

`cargo xtask eu5-web-stage` replaces the ignored directory `solid-csr-spa-template/public/eu5-locations-db/app/` with the host document `index.html`, the generated `pkg/` and the lazily fetched packs. See [vendor instructions](../../../vendor/AGENTS.md) and `tools/xtask/src/eu5_web.rs`.

- **`lang/`.** Game language packs named `<code>.<build>.bitcode.zst`, where the build is the game's Steam build ID. The frame fetches one when the viewer's locale has game text; English is embedded.
- **`fonts/`.** Font packs named `<name>.ttf` for scripts the embedded fonts do not cover (Korean, Japanese, Chinese, Arabic, Devanagari, Bengali, Telugu).
- **Serving.** The frame fetches both from `/eu5-locations-db/app/` in CORS mode without credentials, and the server answers with `Access-Control-Allow-Origin: *`. Language packs carry the build in their name; font packs are served with revalidation. The exact path patterns and size bounds are in the host protocol.

A missing pack degrades to English text or fallback glyphs; it does not break the page. The Docker image must copy the staged `lang/` and `fonts/` directories together with `pkg/`.

## Maintenance

Update the [site map](site-map.md) when the route, guard or frame surface changes. Change the protocol in the application repository first, then the page, in the same release; keep the vendored gitlink moves deliberate.
