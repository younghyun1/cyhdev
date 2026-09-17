import { expect, test } from "@playwright/test";
import { installApiMocks, setUiPreferences } from "./fixtures";

for (const viewport of [{ width: 1440, height: 900 }, { width: 390, height: 844 }]) {
  test(`Minecraft map loads only on its page at ${viewport.width}px`, async ({ page }) => {
    await page.setViewportSize(viewport);
    await installApiMocks(page, "superuser");
    await setUiPreferences(page, "en-US", "dark");
    let requests = 0;
    await page.route("**/minecraft/map/", async (route) => {
      requests += 1;
      await route.fulfill({ contentType: "text/html", body: "<h1>World map</h1>" });
    });
    await page.goto("/");
    await expect(page.locator(".site-header")).toBeVisible();
    expect(requests).toBe(0);
    if (viewport.width < 768) {
      await page.getByRole("button", { name: "Open sidebar menu" }).click();
    }
    await page.getByRole("button", { name: /^Projects/ }).click();
    await page.getByRole("link", { name: "Minecraft Map" }).click();
    await expect(page).toHaveURL(/\/minecraft$/);
    await expect(page.frameLocator(".minecraft-map").getByRole("heading")).toHaveText("World map");
    expect(requests).toBe(1);
    const bounds = await page.locator(".minecraft-map").boundingBox();
    expect(bounds).not.toBeNull();
    expect(bounds!.width).toBeLessThanOrEqual(viewport.width);
    expect(bounds!.height).toBeGreaterThan(viewport.height * 0.7);
    expect(bounds!.y).toBeGreaterThanOrEqual(44);
    expect(bounds!.y + bounds!.height).toBeLessThan(viewport.height);
  });
}
