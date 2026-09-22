import { resolve } from "node:path";
import { expect, test } from "@playwright/test";
import { installApiMocks, setUiPreferences } from "./fixtures";

for (const width of [320, 390, 1440]) {
  for (const theme of ["light", "dark"] as const) {
    test(`statistics tokens, live values, and theme switching ${width} ${theme}`, async ({ page }, testInfo) => {
      await page.setViewportSize({ width, height: 900 });
      await installApiMocks(page, "logged-out");
      await setUiPreferences(page, "en-US", theme);
      await page.routeWebSocket("**/ws/**", (socket) => {
        if (!socket.url().endsWith("/ws/host-stats")) return;
        const frame = Buffer.alloc(20);
        frame.writeFloatBE(42.5, 0);
        frame.writeBigUInt64BE(8n * 1024n ** 3n, 4);
        frame.writeBigUInt64BE(3n * 1024n ** 3n, 12);
        for (let sample = 0; sample < 60; sample += 1) {
          frame.writeFloatBE(sample === 59 ? 42.5 : 35 + Math.sin(sample / 7) * 12, 0);
          socket.send(frame);
        }
      });
      const errors: string[] = [];
      page.on("pageerror", (error) => errors.push(error.message));
      await page.goto("/backend-stats", { waitUntil: "networkidle" });
      await expect(page.getByText("42.5%", { exact: true })).toBeVisible();
      await expect(page.getByText("5.0 GiB / 8.0 GiB", { exact: true })).toBeVisible();
      await expect(page.locator("canvas[role='img']")).toHaveCount(2);
      await expect(page.locator(".stats-connection-dot")).toHaveClass(/is-live/);
      if (theme === "dark") await expect(page.locator(".stats-chart-value").first()).toHaveCSS("color", "rgb(251, 191, 36)");
      const panels = page.locator(".stats-panel");
      await expect(panels).toHaveCount(3);
      for (const panel of await panels.all()) {
        await expect(panel).toHaveCSS("background-image", "none");
        await expect(panel).toHaveCSS("box-shadow", "none");
        await expect(panel).toHaveCSS("border-top-width", "1px");
      }
      await page.evaluate(() => document.fonts.ready);
      await page.screenshot({ path: resolve(import.meta.dirname, `../../target/statistics-polish/after/${testInfo.project.name}/${width}-${theme}.png`), fullPage: true });
      const before = await page.locator("canvas").first().evaluate((element) => (element as HTMLCanvasElement).toDataURL());
      if (width < 768) await page.getByRole("button", { name: "Open sidebar menu" }).click();
      await page.getByRole("button", { name: "Toggle dark/light mode" }).click();
      await expect(page.locator("html")).toHaveClass(theme === "light" ? /dark/ : /light/);
      await expect.poll(() => page.locator("canvas").first().evaluate((element) => (element as HTMLCanvasElement).toDataURL())).not.toBe(before);
      if (width < 768) await page.getByRole("button", { name: "Close", exact: true }).click();
      await expect(page.getByText("42.5%", { exact: true })).toBeVisible();
      expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(width + 1);
      expect(errors).toEqual([]);
    });
  }
}
