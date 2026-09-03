import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  testMatch: "performance.spec.ts",
  timeout: 180_000,
  workers: 1,
  reporter: "line",
  use: {
    baseURL: "http://127.0.0.1:4173",
    trace: "retain-on-failure",
  },
  webServer: {
    command: "npm run build && npm run serve -- --host 127.0.0.1",
    url: "http://127.0.0.1:4173",
    reuseExistingServer: false,
    timeout: 30_000,
  },
  projects: [{ name: "chromium", use: { browserName: "chromium" } }],
});
