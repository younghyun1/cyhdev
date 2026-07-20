/* @refresh reload */
// Runs on the SolidJS 2.0 BETA (solid-js 2.0.0-beta.x + @solidjs/web).
// Prerelease semantics apply: writes flush on a microtask (use flush() for
// sync read-after-write), effects are (compute, apply) pairs, async reads
// suspend to the nearest <Loading>. See README "SolidJS 2.0 beta".
import "./index.css";

import { render } from "@solidjs/web";

import App from "./app";
import { Router } from "@solidjs/router";
import { routes } from "./routes";

const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?",
  );
}

render(() => <Router root={App}>{routes}</Router>, root!);
