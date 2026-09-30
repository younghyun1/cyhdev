import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e/optimization", testMatch: "live.spec.ts", workers: 1, fullyParallel: false,
  retries: 0, timeout: 3_600_000, expect: { timeout: 15_000 }, reporter: "line",
  use: { browserName: "chromium", ignoreHTTPSErrors: true, trace: "off", screenshot: "off", video: "off",
    launchOptions: { args: ["--use-fake-device-for-media-stream", "--use-fake-ui-for-media-stream"] } },
  // The test starts only its supplied binary; there is no Vite server or API interception.
});
