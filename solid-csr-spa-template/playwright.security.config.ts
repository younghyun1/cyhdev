import { defineConfig } from "@playwright/test";

// Runs the production build, not the dev server: the backend's CSP only permits the
// built shell's inline theme script, and dev-server HMR injects other scripts.
export default defineConfig({
  testDir: "./e2e",
  testMatch: "security-headers.spec.ts",
  timeout: 30_000,
  expect: { timeout: 7_500 },
  fullyParallel: true,
  reporter: "line",
  use: {
    baseURL: "http://127.0.0.1:4173",
    trace: "retain-on-failure",
  },
  webServer: {
    command: "npm run build && npm run serve -- --host 127.0.0.1 --port 4173 --strictPort",
    url: "http://127.0.0.1:4173",
    reuseExistingServer: false,
    timeout: 120_000,
  },
  projects: [
    { name: "chromium", use: { browserName: "chromium" } },
    { name: "webkit", use: { browserName: "webkit" } },
  ],
});
