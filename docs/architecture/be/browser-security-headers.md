# Browser security headers and embedded applications

`routers/middleware/browser_policy.rs` adds browser security headers to every response from the HTTPS listener: the SPA shell, static assets, API responses, and errors. It fills only headers a route has not set, so stricter route policies remain, such as `sandbox allow-scripts` on uploaded WASM demo HTML and `no-referrer` on authentication responses.

## Application policy

The Content-Security-Policy is built once at startup in `routers/main_router/http_policy.rs` from configuration:

- `default-src 'self'`, `object-src 'none'`, `base-uri 'self'`, `form-action 'self'`, `frame-ancestors 'self'`, and `frame-src 'self'` for the map, EU5, and demo iframes.
- `script-src 'self'` plus a SHA-256 hash of each inline script in the embedded `index.html`, computed from the file compiled into the binary, so the pre-paint theme script is allowed without `'unsafe-inline'` and a changed script cannot silently drift from the policy.
- `style-src 'self' 'unsafe-inline'`, because the Markdown editor, Leaflet, chart tooltips, and sanitized fastfetch output set inline styles.
- `img-src 'self' data: blob:` plus the media bucket's regional and global S3 origins (bucket from `util/s3.rs`, region from the AWS configuration) and `https://tile.openstreetmap.org`.
- `font-src 'self' data:` for bundled fonts that Vite may inline, and `connect-src 'self'` plus the WebSocket form of `PUBLIC_APP_ORIGIN` and `https://nominatim.openstreetmap.org` for the batch-upload place search.
- No `'wasm-unsafe-eval'`: no page on this origin instantiates WebAssembly; demos and the EU5 app run in sandboxed frames under their own policies.

Every response also gets `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, and `Permissions-Policy` allowing camera and microphone for this origin only. `Strict-Transport-Security: max-age=31536000` is sent outside `CURR_ENV=local`, without `includeSubDomains` because other hosts under the domain are not known to serve HTTPS and a subdomain pin lasts a year. The port-80 redirect listener sends no HSTS, as browsers ignore it over HTTP.

## Embedded applications

The squaremap map under `/minecraft/map/` is written by the Minecraft server's plugin, and the EU5 app under `/eu5-locations-db/app/` is vendored third-party code. Both are served from this origin, so with `allow-same-origin` their scripts could remove the iframe sandbox or call the API with the viewer's cookies. Every response under those paths now carries `Content-Security-Policy: sandbox allow-scripts allow-popups allow-popups-to-escape-sandbox; frame-ancestors 'self'` (EU5 adds `allow-top-navigation-by-user-activation`), which also applies on direct navigation, and the iframes no longer grant `allow-same-origin`. The documents run in an opaque origin: session cookies are not sent, the API's CORS allowlist and trusted-origin checks do not admit the `null` origin, and the frames cannot read the parent document.

An opaque-origin document fetches module scripts, WebAssembly, JSON, and fonts in CORS mode, so responses under both paths send `Access-Control-Allow-Origin: *` without credentials. That exposes nothing new: the files are public and credentialed reads remain impossible.

Inspection of squaremap's web source (upstream `web/src/js/LayerControl.js`) found unguarded `window.localStorage` calls for layer visibility, which throw in an opaque origin and would abort overlay setup. `routers/main_router/squaremap_storage_shim.rs` therefore inserts a bounded in-memory `localStorage` (256 keys) at the start of each squaremap HTML document; layer choices now last for one page view. Because the served HTML differs from the file, HTML responses omit `ETag` and `Last-Modified` and ignore conditional request headers; other map files keep their validators. The copy-link button needs `allow="clipboard-write *"` on the iframe, since the frame is now cross-origin. The EU5 app and its wasm-bindgen output use no Web Storage, IndexedDB, or cookies.

The EU5 host document posts its ready message to `location.origin`, which still names this site in a sandboxed frame, and accepts theme messages whose origin is this site and whose source is its parent. The parent page now accepts the ready signal only when `event.source` is its own frame window and `event.origin` is `"null"`, and it posts the theme with target `"*"` because an opaque origin cannot be named; the payload is only `light` or `dark`. The vendored document needed no change.

Serving the embedded apps from a separate origin, such as a dedicated subdomain, would be stronger still and would let squaremap keep persistent storage; it needs DNS and certificate changes outside this repository.

## Verification

Backend unit tests check the exact policy, source validation, embedded path matching, header precedence, and the inline-script hashing, including CRLF normalization. `browser_check_fixture_matches_the_backend_policy` keeps `solid-csr-spa-template/e2e/security-headers.json` identical to the backend's output for the preview origin and the real `index.html`.

`npm --prefix solid-csr-spa-template run test:e2e:security` builds the frontend, serves it with `vite preview`, applies the fixture headers to every document, and fails on any `securitypolicyviolation` event or CSP console message across every route in the [site map](../../design/fe/site-map.md). The same spec confirms that the harness catches an injected inline script, that a squaremap stand-in runs in the `null` origin with working in-memory storage and no access to the parent, and that the real EU5 host document completes its theme handshake from the sandbox with its module loaded over CORS.
