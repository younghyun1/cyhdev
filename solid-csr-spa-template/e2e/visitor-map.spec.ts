import { expect, test } from "@playwright/test";
import { installApiMocks, setUiPreferences } from "./fixtures";

test("visitor map panes stay behind the mobile navigation drawer", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installApiMocks(page, "superuser");
  await setUiPreferences(page, "en-US", "light");
  await page.goto("/visitor-board");
  await expect(page.locator(".leaflet-control-zoom")).toBeVisible();
  await page.getByRole("button", { name: "Open sidebar menu" }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  const drawerIsOnTop = await dialog.evaluate((element) => {
    const map = document.querySelector(".leaflet-container")?.getBoundingClientRect();
    const drawer = element.getBoundingClientRect();
    if (!map) return false;
    const left = Math.max(drawer.left, map.left);
    const right = Math.min(drawer.right, map.right);
    const top = Math.max(drawer.top, map.top);
    const bottom = Math.min(drawer.bottom, map.bottom);
    return right > left && bottom > top && element.contains(
      document.elementFromPoint((left + right) / 2, (top + bottom) / 2),
    );
  });
  expect(drawerIsOnTop).toBe(true);
  await page.getByRole("button", { name: /^About/ }).click();
  await page.getByRole("link", { name: "About Me", exact: true }).click();
  await expect(page).toHaveURL(/\/about$/);
});
