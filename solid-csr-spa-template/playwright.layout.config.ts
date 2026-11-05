import { defineConfig } from "@playwright/test";
import base from "./playwright.config";

// Loads each route in every language, so it takes minutes and stays out of the main
// suite. Layout does not depend on the browser engine enough to justify a second run.
export default defineConfig({
  ...base,
  testIgnore: undefined,
  testMatch: "layout-stability.spec.ts",
  projects: [{ name: "chromium", use: { browserName: "chromium" } }],
});
