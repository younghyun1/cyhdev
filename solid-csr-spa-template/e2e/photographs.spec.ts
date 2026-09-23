import { expect, test, type Page } from "@playwright/test";
import { installApiMocks, type AuthMode } from "./fixtures";

const ADMIN_CHUNKS = /BatchUploadFields|ProcessingModal|leaflet-geosearch/;

async function requestedModules(page: Page, authMode: AuthMode): Promise<string[]> {
  const urls: string[] = [];
  page.on("request", (request) => urls.push(request.url()));
  await page.setViewportSize({ width: 1440, height: 900 });
  await installApiMocks(page, authMode);
  await page.goto("/photographs", { waitUntil: "networkidle" });
  await expect(page.locator(".photo-card").first()).toBeVisible();
  return urls;
}

test("desktop visitors do not prefetch superuser upload code", async ({ page }) => {
  const urls = await requestedModules(page, "logged-out");
  expect(urls.filter((url) => ADMIN_CHUNKS.test(url))).toEqual([]);
  expect(urls.some((url) => url.includes("PhotographMap"))).toBe(true);
});

test("desktop superusers prefetch upload and processing code", async ({ page }) => {
  const urls = await requestedModules(page, "superuser");
  await expect
    .poll(() => urls.filter((url) => ADMIN_CHUNKS.test(url)).length)
    .toBeGreaterThan(0);
});
