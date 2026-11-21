# SolidJS 2 migration

The frontend uses exact stable SolidJS 2 pins: `solid-js` and `@solidjs/web` 2.0.3, `@solidjs/router` 2.0.1, `@solidjs/vite-plugin` 3.0.1, `@solid-primitives/keyed` 3.0.0, and `@solid-primitives/utils` 7.0.1; `package.json` is authoritative for later versions. The official Vite plugin supplies the Solid 2 native compiler; the compatibility wrapper `vite-plugin-solid` is no longer a direct dependency.

The migration began on release candidates and was promoted to stable releases on November 21, 2026; see the [dependency upgrade plan](../../plans/2026-11-21-dependency-upgrade.md). No source changes were needed for the promotion. `@solidjs/testing-library` remains on 1.0.0-rc.2 until a stable Solid 2 release exists.
