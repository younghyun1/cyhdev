/* @refresh reload */
// Runs on the SolidJS 2.0 release candidate (solid-js + @solidjs/web).
// Prerelease semantics apply: writes flush on a microtask (use flush() for
// sync read-after-write), effects are (compute, apply) pairs, async reads
// suspend to the nearest <Loading>. See README "SolidJS 2.0 release candidate".
// IBM Plex: self-hosted, woff2, font-display swap, unicode-range subset.
// KR package is sliced Google-style; only chunks containing rendered glyphs
// (hero name hangul/hanja) are downloaded. Imported here instead of
// index.css because @tailwindcss/postcss inlines CSS @imports without
// rebasing their relative font URLs (see note in index.css).
import "@fontsource/ibm-plex-sans/400.css";
import "@fontsource/ibm-plex-sans/400-italic.css";
import "@fontsource/ibm-plex-sans/500.css";
import "@fontsource/ibm-plex-sans/600.css";
import "@fontsource/ibm-plex-sans/700.css";
import "@fontsource/ibm-plex-mono/400.css";
import "@fontsource/ibm-plex-mono/500.css";
import "@fontsource/ibm-plex-mono/700.css";
import "@fontsource/ibm-plex-sans-kr/700.css";
import "./index.css";

import { render } from "@solidjs/web";

import { createRouter } from "@solidjs/router";

import App from "./app";
import { routes } from "./routes";

const Router = createRouter({ routes });

const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?",
  );
}

render(
  () => (
    <Router>
      {(props) => <App>{props.children}</App>}
    </Router>
  ),
  root!,
);
