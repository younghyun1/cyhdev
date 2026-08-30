# UI Internationalization Plan

## Goal

Use the existing internationalization direction for fixed UI text, starting with English and Korean.

Initial scope:

- TopBar navigation, site title, auth/menu labels, aria labels, and language selector text.
- BottomBar status labels and mobile status modal labels.
- Common page titles, placeholders, button labels, empty states, and static UI copy.
- Error boundary copy in `App`.

Out of scope for the first pass:

- User-generated blog posts, comments, photograph comments, WASM module titles/descriptions, and other persisted content.
- Full prose pages such as long biography sections unless the text is already acting as a UI heading or label.
- Database-managed editorial translations.

## Baseline Before This Pass

Backend:

- `rust-be-template` already has an i18n table and cache:
  - `migrations/2025-04-29-005518_i18n`
  - `src/domain/i18n/i18n.rs`
  - `src/domain/i18n/i18n_cache.rs`
  - `src/init/state/server_state.rs`
- Startup calls `state.sync_i18n_data().await?` in `src/init/server_init.rs`.
- The existing public route is `GET /api/i18n/country-language-bundle`.
- That route currently extracts `Json<GetCountryLanguageBundleRequest>` from a `GET` request and returns raw `application/octet-stream`.
- The Solid API wrapper currently treats the same endpoint as JSON `ApiResponse<GetCountryLanguageBundleResponse>`, so the existing frontend binding is not directly usable for UI text.
- Existing ISO codes include:
  - English: `language_code = 41`, `country_code = 840` for US.
  - Korean: `language_code = 86`, `country_code = 410` for KR.

Frontend:

- `solid-csr-spa-template/src/services/all_api.ts` has an `i18nApi` placeholder, but UI components do not consume it.
- Fixed strings are currently inline in components and pages.
- `TopBar.tsx` owns the nav link labels, dark-mode button, auth links, profile menu, mobile drawer title, and mobile page title fallback.
- `BottomBar.tsx` owns status labels such as `Site status`, `Close`, `FE`, `BE`, `metrics`, `up`, `handled`, `responses`, `sessions`, `db`, and `tap`.
- There is no existing frontend i18n state module.

## Design Direction

Use the existing `i18n_strings` table as the source of truth for fixed UI text, then hydrate those rows into the existing backend RAM cache on startup. The frontend should request a selected language bundle from the backend, and the backend should serve it from RAM without hitting Postgres on normal page loads.

This aligns with the system that already exists: UI copy is persisted in Postgres, synchronized into `ServerState.i18n_cache` during startup, and can later be edited/administered without recompiling the backend.

Implementation update:

- English and Korean source bundles live in `rust-be-template/i18n/ui/*.json`.
- The backend validates those files at compile/startup through `source_bundles()`.
- Startup calls `sync_file_backed_ui_text_sources()` before `sync_i18n_data()`, so file-backed UI strings are upserted into `i18n_strings` first, then loaded into `ServerState.i18n_cache`.
- The admin i18n sync route also runs the file-backed source sync before rehydrating the RAM cache.
- Normal frontend bundle requests use `GET /api/i18n/ui-text?locale=...`, which reads from RAM only.

Implemented backend files include:

```text
rust-be-template/i18n/ui/en-US.json
rust-be-template/i18n/ui/ko-KR.json
rust-be-template/src/domain/i18n/ui_text/keys.rs
rust-be-template/src/domain/i18n/ui_text/locale.rs
rust-be-template/src/domain/i18n/ui_text/source.rs
rust-be-template/src/dto/responses/i18n/ui_text_bundle_response.rs
rust-be-template/src/handlers/i18n/get_ui_text_bundle.rs
```

Implemented frontend files include:

```text
solid-csr-spa-template/src/i18n/keys.ts
solid-csr-spa-template/src/i18n/defaults/en-us.ts
solid-csr-spa-template/src/state/i18n.ts
solid-csr-spa-template/src/components/LanguageSelect.tsx
```

## Backend Plan

1. Add typed locale support for UI strings.

   Proposed enum:

   ```rust
   pub enum UiLocale {
       EnUs,
       KoKr,
   }
   ```

   It should parse only supported locale strings such as `en-US`, `en`, `ko-KR`, and `ko`. Unknown values should fall back to `EnUs` without panicking.

2. Add centralized UI text key definitions.

   Keep keys stable and namespaced. They should be defined in Rust for handler/cache validation and mirrored in TypeScript for typed frontend access:

   ```rust
   pub const TOP_BAR_SITE_TITLE: &str = "top_bar.site_title";
   pub const TOP_BAR_NAV_HOME: &str = "top_bar.nav.home";
   pub const TOP_BAR_NAV_ABOUT: &str = "top_bar.nav.about";
   pub const BOTTOM_BAR_SITE_STATUS: &str = "bottom_bar.site_status";
   pub const PAGE_LOGIN_TITLE: &str = "page.login.title";
   ```

   These constants are keys only, not translation storage. Do not store translated UI copy in Rust constants.

3. Seed English and Korean UI text in the database.

   Commit English and Korean JSON source bundles and upsert them into `i18n_strings` during startup/admin sync for:

   - `en-US`: `language_code = 41`, `country_code = 840`.
   - `ko-KR`: `language_code = 86`, `country_code = 410`.

   The sync path uses the stable reference keys from the key list:

   ```rust
   state.sync_file_backed_ui_text_sources().await?;
   state.sync_i18n_data().await?;
   ```

   The sync first updates matching rows with `country_subdivision_code IS NULL`; if no row exists, it inserts one. This is intentionally idempotent even though the existing nullable-column unique constraint cannot make `ON CONFLICT` reliable for `NULL` subdivision rows.

4. Serve UI text from the startup-hydrated RAM cache.

   `server_init_proc` now calls:

   ```rust
   state.sync_file_backed_ui_text_sources().await?;
   state.sync_i18n_data().await?;
   ```

   Keep using that path. The new UI bundle handler should read `state.i18n_cache` and filter cached rows by country, language, and UI key namespace. It should not query Postgres during normal requests.

   Prefer adding an `I18nCache` helper such as:

   ```rust
   pub fn ui_text_bundle(
       &self,
       country_code: i32,
       language_code: i32,
       keys: &[&'static str],
   ) -> HashMap<String, String>
   ```

   Avoid N+1 database work entirely; all lookups should happen against the in-memory indexes or cached rows.

5. Expose a JSON endpoint for UI text.

   Proposed endpoint:

   ```text
   GET /api/i18n/ui-text?locale=en-US
   ```

   Proposed response:

   ```ts
   interface UiTextBundleResponse {
     locale: "en-US" | "ko-KR";
     fallback_locale: "en-US";
     texts: Record<UiTextKey, string>;
   }
   ```

   This should return through the existing `ApiResponse` convention, unlike the current binary country-language bundle endpoint.

6. Keep the existing binary country/language bundle endpoint separate.

   Do not force the Solid app to decode Rust `bitcode` in the browser for the UI string pass. That endpoint can remain for future non-UI i18n work, but it should eventually be corrected because `GET` plus JSON body is awkward and the generated frontend wrapper currently mismatches the raw byte response.

7. Startup validation.

   Add a cheap validation function that checks:

   - `en-US` and `ko-KR` have the same keys.
   - No duplicate keys exist inside a locale in the cached rows.
   - Every required UI key is represented in the default locale.
   - Missing Korean keys are logged and can fall back to English.

   Log validation failures with structured `tracing` fields. Missing default English keys should fail startup because fallback would be unsafe. Missing Korean keys can be warnings if the response fills them from English fallback.

## Frontend Plan

1. Add typed i18n state.

   `src/state/i18n.ts` should own:

   - Supported locale union: `"en-US" | "ko-KR"`.
   - Current locale signal.
   - Loaded translation map signal.
   - `t(key: UiTextKey): string`.
   - `setLocale(locale)` with localStorage persistence.
   - `loadUiTextBundle(locale)` that calls the backend and falls back to bundled TS defaults on failure.

   No `any`; all maps should use `Record<UiTextKey, string>`.

2. Add frontend defaults.

   Keep a minimal TypeScript English default bundle in `src/i18n/defaults` so the app still renders if the backend request fails or if the first render happens before the request resolves.

   This is only an emergency client fallback. The authoritative copy lives in the database and backend RAM cache.

3. Add the language selector beside the dark-mode button.

   In `TopBar.tsx`, insert a compact `LanguageSelect` immediately next to the existing theme toggle in both authenticated and unauthenticated right-side controls.

   UI behavior:

   - Options: `English` and `한국어`.
   - Persist selection in `localStorage`, for example `ui_locale`.
   - Update `<html lang="en">` / `<html lang="ko">`.
   - Do not require login.
   - Keep it keyboard accessible with a real `select`.

4. Replace TopBar strings first.

   Convert:

   - `NAV_LINKS` labels.
   - `titleFromPath` fallback for known routes.
   - Site title.
   - `Login`, `Edit Profile`, `Logout`, `Menu`.
   - Aria labels for sidebar, theme toggle, and profile menu.

5. Replace BottomBar strings second.

   Convert fixed labels while preserving dynamic values:

   - `Site status`, `Close`, `FE`, `BE`, `built`, `w. solidjs`.
   - `metrics`, `up`, `handled`, `responses`, `sessions`.
   - `db`, `db latency`, `time to generate state report`, `net`, `state`, `tap`.

   Duration formatting such as `1d 2h 3m` can stay language-neutral for the first pass.

6. Replace page titles and common controls.

   Prioritize:

   - `app.tsx` error boundary.
   - `pages/live_chat.tsx` and `components/LiveChatPanel.tsx`.
   - Auth pages: login, signup, find password, reset password, edit profile.
   - Blog post list/new/edit controls.
   - Photograph/project modal labels and action labels.
   - Geo IP page headings and placeholders.

7. Do not translate user content.

   Keep dynamic values as-is:

   - Blog post titles and bodies.
   - Comments.
   - Photograph comments.
   - User names and emails.
   - Project titles/descriptions from the backend.

## Suggested Initial Key Groups

```text
common.close
common.cancel
common.save
common.delete
common.edit
common.upload
common.loading
common.error

top_bar.site_title
top_bar.nav.home
top_bar.nav.about
top_bar.nav.about_blog
top_bar.nav.blog
top_bar.nav.photographs
top_bar.nav.projects
top_bar.nav.visitor_board
top_bar.nav.geo_ip
top_bar.nav.backend_stats
top_bar.auth.login
top_bar.auth.logout
top_bar.profile.edit
top_bar.menu.title
top_bar.aria.open_sidebar
top_bar.aria.toggle_theme
top_bar.aria.open_user_menu
top_bar.language.label
top_bar.language.english
top_bar.language.korean

bottom_bar.site_status
bottom_bar.fe
bottom_bar.be
bottom_bar.built
bottom_bar.with_solid
bottom_bar.metrics
bottom_bar.up
bottom_bar.handled
bottom_bar.responses
bottom_bar.sessions
bottom_bar.db
bottom_bar.db_latency
bottom_bar.state_age
bottom_bar.net
bottom_bar.tap

page.home.title
page.about.title
page.about_blog.title
page.blog.list_title
page.blog.new_title
page.blog.edit_title
page.live_chat.title
page.photographs.title
page.projects.title
page.geo_ip.title
page.backend_stats.title
page.login.title
page.signup.title
page.find_password.title
page.reset_password.title
page.edit_profile.title
page.not_found.title
```

## Implementation Order

1. Backend key registry and endpoint.

   Add Rust key constants, DTO, cache helper, handler, route, and OpenAPI registration. Keep all new error handling explicit with `match`; do not introduce `unwrap()` or `expect()`.

2. File-backed database seed sync.

   Keep English and Korean rows in committed JSON source files, validate all required keys, then upsert them into `i18n_strings` during startup/admin sync before RAM cache hydration. This avoids duplicating the translation body in SQL while still making Postgres the durable runtime source.

3. Frontend i18n state and service.

   Add the typed key union, default bundles, locale state, loader, and API wrapper for `/api/i18n/ui-text`.

4. TopBar language selector and string replacement.

   Add the selector beside the theme button and replace TopBar strings with `t(key)`.

5. BottomBar replacement.

   Replace status labels and mobile modal labels.

6. Page title pass.

   Replace high-signal page titles and common controls, leaving long prose and user content alone.

7. Tests and checks.

   Frontend:

   ```text
   npm run typecheck
   npm run lint
   npm run test
   ```

   Backend:

   ```text
   cargo fmt
   cargo check
   cargo clippy
   ```

## Risks

- The existing binary i18n endpoint should not be reused directly by the browser until the response/client mismatch is fixed.
- Maintaining database seed rows and TypeScript key unions manually can drift. The first pass should include validation tests, and a later pass can generate key unions from a neutral source.
- Some Korean labels may need copy review. Use clear, plain UI Korean first instead of over-translating technical status labels.
- Replacing every inline string at once would be noisy. The first implementation should prefer layout, navigation, page titles, and common controls.
- If the database has no seeded default English UI rows, startup should fail loudly instead of serving a half-translated shell.

## Acceptance Criteria

- A language selector appears immediately next to the dark-mode button.
- The selected language persists across reloads.
- TopBar, BottomBar, and initial page titles switch between English and Korean without a full reload.
- Fixed UI strings are persisted in `i18n_strings`, loaded into RAM during startup, and served from `ServerState.i18n_cache`.
- Frontend access is typed through `UiTextKey`; no `any` is introduced.
- Backend serves the UI bundle without querying Postgres during normal requests.
- Existing database-backed i18n code remains intact for future content translation work.
