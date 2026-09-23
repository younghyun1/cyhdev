# Frontend

The application uses Solid 2 prereleases, not Solid 1 or React semantics. `package.json` and `package-lock.json` are authoritative for versions; the migration document contains older pins. Preserve the coordinated exact Solid, web, router, and compiler pins and `.npmrc` compatibility settings unless dependency work is explicitly scoped.

## Entry points and conventions

- Start page and navigation work with the [complete page flowcharts and route inventory](../docs/design/fe/site-map.md). Keep its diagrams and inventory synchronized with route, guard, menu, and embedded-surface changes.
- `src/index.tsx` mounts the router; `src/routes.ts` owns routes; `src/app.tsx` owns the shell. Extend `src/pages/`, `src/components/`, and existing `src/state/` owners. Read nearby Solid 2 examples before changing effects, async reads, or synchronous read-after-write behavior.
- Use strict TypeScript, the Solid router, and context for new shared state where appropriate. Preserve existing state ownership instead of introducing competing stores. JSX types come from `@solidjs/web` in this stack.
- Centralize styles in `src/index.css` and `src/styles/`; follow [design system](../docs/design/fe/design-system.md) and [mobile design](../docs/design/fe/mobile.md). Preserve route-level lazy loading and initial-asset budgets in `build/initialAssetBudget.ts`.
- Read the corresponding document under `docs/design/fe/` for account, admin, forum, and call UI changes. Check keyboard interaction, narrow layouts, loading/error states, and authorization-dependent controls.

## API ownership

`src/services/api.ts` owns `apiFetch`, cookie credentials, build headers, and unauthorized-session handling. Feature wrappers in `src/services/contracts/` adapt generated clients. Preserve progress-aware upload behavior in `src/services/upload_with_progress.ts`. `VITE_API_URL` defaults to same-origin; Vite does not configure a backend proxy.

Read [API contracts](../docs/design/fe/api-contracts.md) before adding HTTP calls. `src/generated/` is generated from Rust OpenAPI and must not be edited manually or shadowed with handwritten HTTP DTOs. Update the backend source and run the root generation command in backend instructions. Persistent/binary protocols such as chat and RTC retain their explicitly typed protocol modules.

## Commands

From the root: `npm --prefix solid-csr-spa-template ci`, then `cargo xtask frontend`. The dev server listens on port 3000. Final verification is `cargo xtask frontend-check`; it includes typecheck, zero-warning lint, Vitest, and the Vite build, but not Playwright. For focused tests, use `npm --prefix solid-csr-spa-template run test -- <test-file>`.

For browser behavior, run `npm --prefix solid-csr-spa-template run test:e2e:chromium`, or `test:e2e` for Chromium and WebKit, with installed Playwright browsers. The config starts a local dev server; inspect each test's fixtures before assuming a backend is required. Performance tests use the separate `test:e2e:performance` script, and `test:e2e:security` checks the built frontend under the backend's Content-Security-Policy. Record browser checks actually run.

Keep `dist/`, `node_modules/`, and staged `public/eu5-locations-db/app/` untracked. EU5 staging uses root `cargo xtask eu5-web-stage`; consult [vendor instructions](../vendor/AGENTS.md).
