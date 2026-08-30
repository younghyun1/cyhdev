## SolidJS 2.0 release candidate

This project runs on the SolidJS 2.0 release candidate (`solid-js@2.0.0-rc.x` + `@solidjs/web`), with the matching prerelease ecosystem: `@solidjs/router@2.0.0-next`, `vite-plugin-solid@3.0.0-next`, `@solid-primitives/keyed@3.0.0-next`, `@solidjs/testing-library@1.0.0-beta`. Expect churn until 2.0 stable; pin exact versions when bumping.

Prerelease workarounds to revisit at 2.0 stable:
- `.npmrc` sets `legacy-peer-deps`: strict `>=2.0.0` peer ranges reject prerelease versions.
- `@solid-primitives/utils` is a direct dependency only because `@solid-primitives/keyed`'s next build forgets to declare it.
- `eslint-plugin-solid` 0.16 supports Solid 2, but its `solid/imports` rule still maps `JSX` to `solid-js`; the RC exports that type from `@solidjs/web`, so the rule remains disabled.
- TypeScript 7 supplies `tsc` through the `@typescript/native` npm alias; the root `typescript` alias exposes the TypeScript 6 API and `tsc6` for tools such as `typescript-eslint` that still require programmatic compiler access.
- `src/components/LineChart.tsx` replaces `solid-chartjs`, which pins solid-js 1.x.

## Usage

Dependencies are installed from the committed npm lockfile. Run frontend commands from the monorepo root so local and image builds use the same paths and inputs.

```bash
$ cargo xtask frontend
$ cargo xtask frontend-build
```

## Exploring the template

This template's goal is to showcase the routing features of Solid.
It also showcase how the router and Suspense work together to parallelize data fetching tied to a route via the `.data.ts` pattern.

You can learn more about it on the [`@solidjs/router` repository](https://github.com/solidjs/solid-router)

### Learn more on the [Solid Website](https://solidjs.com) and come chat with us on our [Discord](https://discord.com/invite/solidjs)

## Available commands

Run these from the monorepo root.

### `cargo xtask frontend`

Runs the app in the development mode.<br>
Open [http://localhost:3000](http://localhost:3000) to view it in the browser.

The page will reload if you make edits.<br>

### `cargo xtask frontend-build`

Builds the app for production to the `dist` folder.<br>
It correctly bundles Solid in production mode and optimizes the build for the best performance.

The build is minified and the filenames include the hashes.<br>
Your app is ready to be deployed!

## Deployment

You can deploy the `dist` folder to any static host provider (netlify, surge, now, etc.)

## This project was created with the [Solid CLI](https://solid-cli.netlify.app)
