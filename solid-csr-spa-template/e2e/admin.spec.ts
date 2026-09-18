import { expect, test } from "@playwright/test";
import { installApiMocks, setUiPreferences } from "./fixtures";

for (const width of [390, 768, 1440]) {
  test(`Admin navigation stays reachable while scrolling at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    await installApiMocks(page, "superuser");
    await setUiPreferences(page, "en-US", "dark");
    await page.goto("/admin/operations");
    await expect(page.locator("#hard-purge")).toBeVisible();
    await page.evaluate(() => window.scrollTo(0, 800));
    const nav = page.locator(width < 768 ? ".admin-workspace-mobile-navigation" : ".admin-workspace-sidebar-inner");
    await expect(nav).toBeInViewport();
    const box = await nav.boundingBox();
    const header = await page.locator(".site-header").boundingBox();
    expect(box!.y).toBeGreaterThanOrEqual(header!.height - 1);
    await nav.locator('a[href="/admin/operations#hard-purge"]').click();
    await expect(page).toHaveURL(/#hard-purge$/);
    const panel = await page.locator("#hard-purge").boundingBox();
    expect(panel!.y).toBeGreaterThanOrEqual(header!.height);
    expect(panel!.y).toBeLessThan(800);
    await nav.locator('a[href="/admin/authorization"]').click();
    await expect(page.locator(".authorization-page")).toBeVisible();
    await nav.locator('a[href="/admin/operations#media-cleanup"]').click();
    await expect(page).toHaveURL(/#media-cleanup$/);
    await expect.poll(async () => (await page.locator("#media-cleanup").boundingBox())?.y).toBeLessThan(800);
    await page.screenshot({ path: test.info().outputPath("admin-navigation.png") });
  });
}
