import { expect, test } from "@playwright/test";
import { installApiMocks, setUiPreferences } from "./fixtures";

for (const width of [320, 390, 1440]) {
  test(`search alignment and status metadata at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    await installApiMocks(page, "superuser");
    await setUiPreferences(page, "en-US", "light");
    await page.goto("/forum");
    const input = page.locator(".forum-search input");
    const button = page.locator(".forum-search button[type=submit]");
    await expect(input).toBeVisible();
    const inputBox = await input.boundingBox();
    const buttonBox = await button.boundingBox();
    expect(inputBox).not.toBeNull();
    expect(buttonBox).not.toBeNull();
    expect(inputBox!.height).toBe(buttonBox!.height);
    if (width >= 768) expect(inputBox!.y).toBe(buttonBox!.y);
    const copyright = page.locator("[data-site-bar=bottom] .site-copyright");
    await expect(copyright).toBeVisible();
    await expect(copyright.locator("span")).toHaveText([
      `© 2025-${new Date().getUTCFullYear()} Young Hyun Chi.`,
      "Code licensed under the MIT License.",
    ]);
    const copyrightBox = await copyright.boundingBox();
    expect(copyrightBox!.x).toBeGreaterThanOrEqual(0);
    expect(copyrightBox!.x + copyrightBox!.width).toBeLessThanOrEqual(width);
    await page.screenshot({ path: `../target/page-polish/copyright-${width}.png`, fullPage: true });
    if (width < 768) await page.locator("[data-site-bar=bottom]").click();
    const status = width < 768 ? page.getByRole("dialog") : page.locator("[data-site-bar=bottom]");
    await expect(status).toContainText(/FE · built .* UTC · SolidJS .* · TypeScript .* · Vite/);
    await expect(status).toContainText(/BE · built .* UTC · Axum test · Rust 1.test/);
    await expect(status).toContainText("PostgreSQL test");
    await expect(status).not.toContainText("PostgreSQL PostgreSQL");
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(width);
    await page.screenshot({ path: `../target/page-polish/forum-${width}.png`, fullPage: true });
  });
}

test("visitor counts render as plain text", async ({ page }) => {
  await installApiMocks(page, "logged-out");
  await setUiPreferences(page, "en-US", "light");
  await page.route("**/api/visitor-board", (route) => route.fulfill({ json: { data: [[[37.5, 127], 6]] } }));
  await page.goto("/visitor-board");
  const popup = page.locator(".leaflet-popup-content");
  await expect(popup).toBeVisible();
  await expect(popup).not.toContainText("<b>");
  await expect(popup).toContainText(/Visitations from here: \d+/);
});

test("About interests remain English with Korean navigation", async ({ page }) => {
  await installApiMocks(page, "logged-out");
  await setUiPreferences(page, "ko-KR", "light");
  await page.goto("/about");
  await expect(page.locator("html")).toHaveAttribute("lang", "ko-KR");
  await expect(page.locator(".about-page")).toHaveAttribute("lang", "en");
  await expect(page.getByRole("heading", { name: "5. Volunteer work & Interests" })).toBeVisible();
  await expect(page.getByText("Taught English at a community center.")).toBeVisible();
  await expect(page.locator(".about-page")).not.toContainText(/West Papua|human rights|Sakartvelo/);
});

for (const width of [320, 1440]) {
  test(`comment sort label stays on one line at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    await installApiMocks(page, "superuser");
    await setUiPreferences(page, "en-US", "light");
    await page.goto("/blog/11111111-1111-4111-8111-111111111111");
    const label = page.locator(".blog-comments-heading label span");
    await expect(label).toBeVisible();
    const lines = await label.evaluate((element) => {
      const range = document.createRange();
      range.selectNodeContents(element);
      return range.getClientRects().length;
    });
    expect(lines).toBe(1);
    await label.scrollIntoViewIfNeeded();
    await page.screenshot({ path: `../target/page-polish/blog-${width}.png` });
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(width);
  });
}
