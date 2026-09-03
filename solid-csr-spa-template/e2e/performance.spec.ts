import { expect, test } from "@playwright/test";
import { installApiMocks, setUiPreferences } from "./fixtures";

type Metrics = {
  readonly cls: number;
  readonly interaction: number;
  readonly lcp: number;
};

const routes = [
  "/",
  "/blog",
  "/forum",
  "/photographs",
  "/login",
  "/live-chat",
] as const;

const median = (values: readonly number[]): number => {
  const ordered = [...values].sort((left, right) => left - right);
  return ordered[Math.floor(ordered.length / 2)] ?? Number.POSITIVE_INFINITY;
};

test("five-run cold-cache mobile medians meet delivery targets", async ({
  browser,
}) => {
  const samples = new Map<string, Metrics[]>();

  for (const path of routes) {
    const routeSamples: Metrics[] = [];
    for (let run = 0; run < 5; run += 1) {
      const context = await browser.newContext({
        viewport: { width: 390, height: 844 },
      });
      const page = await context.newPage();
      await installApiMocks(page, "logged-out");
      await setUiPreferences(page, "en-US", "light");
      await page.addInitScript(() => {
        const metrics = { cls: 0, interaction: 0, lcp: 0 };
        Object.defineProperty(window, "__mobileMetrics", {
          configurable: true,
          value: metrics,
        });
        new PerformanceObserver((list) => {
          const entries = list.getEntries();
          const last = entries[entries.length - 1];
          if (last) metrics.lcp = last.startTime;
        }).observe({ type: "largest-contentful-paint", buffered: true });
        new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            const shift = entry as PerformanceEntry & {
              hadRecentInput: boolean;
              value: number;
            };
            if (!shift.hadRecentInput) metrics.cls += shift.value;
          }
        }).observe({ type: "layout-shift", buffered: true });
        new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            const interaction = entry as PerformanceEntry & {
              interactionId: number;
            };
            if (interaction.interactionId > 0) {
              metrics.interaction = Math.max(
                metrics.interaction,
                interaction.duration,
              );
            }
          }
        }).observe({ type: "event", durationThreshold: 16 });
      });

      const session = await context.newCDPSession(page);
      await session.send("Network.enable");
      await session.send("Network.clearBrowserCache");
      await session.send("Network.emulateNetworkConditions", {
        offline: false,
        latency: 150,
        downloadThroughput: 200 * 1024,
        uploadThroughput: 75 * 1024,
        connectionType: "cellular4g",
      });
      await session.send("Emulation.setCPUThrottlingRate", { rate: 4 });

      await page.goto(path, { waitUntil: "load" });
      await page.waitForTimeout(750);
      const menu = page.getByRole("button", {
        name: /navigation|sidebar/i,
      });
      await menu.click();
      await expect(page.getByRole("dialog")).toBeVisible();
      await page.keyboard.press("Escape");
      await page.waitForTimeout(250);
      const metrics = await page.evaluate(
        () =>
          (
            window as typeof window & {
              __mobileMetrics: Metrics;
            }
          ).__mobileMetrics,
      );
      routeSamples.push(metrics);
      await context.close();
    }
    samples.set(path, routeSamples);
  }

  for (const [path, routeSamples] of samples) {
    const result = {
      cls: median(routeSamples.map((sample) => sample.cls)),
      interaction: median(
        routeSamples.map((sample) => sample.interaction),
      ),
      lcp: median(routeSamples.map((sample) => sample.lcp)),
    };
    console.info(`${path} mobile medians`, result);
    expect(result.lcp, `${path} median LCP`).toBeLessThanOrEqual(2_500);
    expect(
      result.interaction,
      `${path} median interaction duration`,
    ).toBeLessThanOrEqual(200);
    expect(result.cls, `${path} median CLS`).toBeLessThanOrEqual(0.1);
  }
});
