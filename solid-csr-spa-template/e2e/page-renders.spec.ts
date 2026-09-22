import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, test } from "@playwright/test";
import { installApiMocks, setUiPreferences } from "./fixtures";

const id = "11111111-1111-4111-8111-111111111111";
const map = readFileSync(resolve(import.meta.dirname, "../../docs/design/fe/site-map.md"), "utf8");
const routes = [...map.matchAll(/^\| `([^`]+)` \| (Public|Signed in|Superuser|Fallback) \|/gm)]
  .map((match) => match[1] ?? "/404");

for (const viewport of [{ width: 1440, height: 900 }, { width: 390, height: 844 }]) {
  for (const theme of ["light", "dark"] as const) {
    for (const pattern of routes) {
      const path = pattern.replace(/:post_id|:topic_id|:photograph_id/g, id)
        .replace(":userName", "mobile-superuser").replace("*404", "/missing-page-fixture");
      const name = pattern === "*404" ? "unmatched-route" : pattern.replace(/[^a-z0-9]+/gi, "-").replace(/^-|-$/g, "") || "home";
      test(`render ${name} ${viewport.width} ${theme}`, async ({ page }, testInfo) => {
        await page.setViewportSize(viewport);
        await installApiMocks(page, /^\/(login|register|find-password|reset-password|verify-email)$/.test(path) ? "logged-out" : "superuser");
        await setUiPreferences(page, "en-US", theme);
        await page.route("**/api/forum/notifications**", (route) => route.fulfill({ json: { data: { notifications: [], next_cursor: null } } }));
        await page.route("**/api/admin/minecraft", (route) => route.fulfill({ json: { data: { players: [{ id, name: "Alex", map_hidden: false }], whitelist: [], whitelist_enabled: true } } }));
        await page.route("**/api/live-chat/messages**", (route) => route.fulfill({ json: { data: { items: [], has_more: false, next_before_message_id: null } } }));
        // Embedded services are not available in the local UI fixture environment.
        for (const surface of ["**/minecraft/map/", "**/eu5-locations-db/app/index.html"]) {
          await page.route(surface, (route) => route.fulfill({ status: 503, contentType: "text/html", body: "<!doctype html><p>Embedded service unavailable in local preview.</p>" }));
        }
        await page.routeWebSocket("**/ws/**", (socket) => {
          socket.onMessage(() => {});
        });
        const errors: string[] = [];
        page.on("pageerror", (error) => errors.push(error.message));
        await page.goto(path, { waitUntil: "networkidle" });
        await expect(page).toHaveTitle("Young Hyun Chi | Software Engineer");
        await expect(page.locator("[data-site-bar='top']")).toBeVisible();
        await page.evaluate(() => document.fonts.ready);
        const screenshot = resolve(import.meta.dirname, `../../target/page-renders/${viewport.width}-${theme}/${name}.png`);
        await page.screenshot({ path: screenshot, fullPage: true, animations: "disabled" });
        await testInfo.attach("page-render", { path: screenshot, contentType: "image/png" });
        expect(errors).toEqual([]);
        expect(await page.evaluate(() => document.documentElement.scrollWidth))
          .toBeLessThanOrEqual(viewport.width + 1);
      });
    }
  }
}
