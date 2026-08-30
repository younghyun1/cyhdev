# SolidJS 2 migration

The frontend uses exact SolidJS 2 release-candidate pins rather than a mutable branch: `solid-js` and `@solidjs/web` 2.0.0-rc.4, `@solidjs/router` 2.0.0-next.19, and `@solidjs/vite-plugin` 3.0.0-next.36. The official Vite plugin supplies the Solid 2 native compiler; the compatibility wrapper `vite-plugin-solid` is no longer a direct dependency.

These are intentionally exact versions because SolidJS 2 has not reached a stable release. The source is migrated to the Solid 2 APIs and the root frontend/final-review commands retain type, lint, test, and build proof. Wave 8 executes that proof. After upstream publishes stable SolidJS 2 and compatible router/plugin releases, update all four pins together and remove the external promotion gate from the root TODO.
