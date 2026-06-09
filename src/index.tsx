/* @refresh reload */
import "./index.css";

import { render } from "solid-js/web";
import { Chart, Filler } from "chart.js";

import App from "./app";
import { Router } from "@solidjs/router";
import { routes } from "./routes";

// Register the Filler plugin once so CpuStatsCard/RamStatsCard area gradients (fill: true) render.
Chart.register(Filler);

const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?",
  );
}

render(() => <Router root={App}>{routes}</Router>, root!);
