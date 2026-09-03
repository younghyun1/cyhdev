import { expect, test } from "@playwright/test";
import { installApiMocks, setUiPreferences } from "./fixtures";

for (const viewport of [
  { width: 768, height: 1024 },
  { width: 1024, height: 768 },
  { width: 1440, height: 900 },
] as const) {
  test(`desktop shell remains active at ${viewport.width}x${viewport.height}`, async ({
    page,
  }) => {
    await page.setViewportSize(viewport);
    await installApiMocks(page, "superuser");
    await setUiPreferences(page, "en-US", "dark");
    await page.goto("/");
    await expect(page.getByText("Younghyun's Blog", { exact: true })).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Open sidebar menu" }),
    ).toBeHidden();
    const layout = await page.evaluate(() => ({
      documentWidth: document.documentElement.scrollWidth,
      viewportWidth: document.documentElement.clientWidth,
    }));
    expect(layout.documentWidth).toBeLessThanOrEqual(layout.viewportWidth);
  });
}

test("desktop navigation and profile menu retain their interactions", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await installApiMocks(page, "superuser");
  await page.goto("/");
  await page.getByRole("button", { name: "About" }).click();
  await expect(page.getByRole("link", { name: "About Me" })).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("link", { name: "About Me" })).toBeHidden();

  const profile = page.getByRole("button", { name: "Open user menu" });
  await profile.click();
  const menu = page.getByRole("menu");
  await expect(menu).toBeVisible();
  await expect(menu.getByRole("menuitem", { name: "Edit Profile" })).toBeVisible();
  await page.mouse.click(600, 300);
  await expect(menu).toBeHidden();
});

test("desktop editor, modal, table, chart, and authentication views stay functional", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await installApiMocks(page, "superuser");

  await page.goto("/blog/new");
  await expect(page.locator(".toastui-editor-defaultUI")).toBeVisible();
  await expect(page.locator(".toastui-editor-md-preview")).toBeVisible();

  await page.goto("/projects");
  await page.locator(".project-card").click();
  const projectDialog = page.getByRole("dialog");
  await expect(projectDialog).toBeVisible();
  await expect(page.getByRole("link", { name: "Open separately" })).toBeHidden();
  await page.keyboard.press("Escape");
  await expect(projectDialog).toBeHidden();

  await page.goto("/admin/authorization");
  await expect(page.locator(".authorization-table").first()).toHaveCSS(
    "display",
    "table",
  );
  await page.goto("/backend-stats");
  await expect(page.locator("canvas").first()).toBeVisible();
});

test("desktop authentication focus keeps status visible and EU5 stays eager", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1024, height: 768 });
  await installApiMocks(page, "logged-out");
  await page.goto("/login");
  await page.getByPlaceholder("Email").focus();
  await expect(page.locator("[data-site-bar='bottom']")).toBeVisible();

  await page.goto("/eu5-locations-db");
  const frame = page.locator(".eu5-locations-db-frame");
  await expect(frame).toHaveAttribute("loading", "eager");
  const bars = await page.evaluate(() => {
    const frameRect = document
      .querySelector(".eu5-locations-db-page")!
      .getBoundingClientRect();
    const top = document
      .querySelector('[data-site-bar="top"]')!
      .getBoundingClientRect();
    const bottom = document
      .querySelector('[data-site-bar="bottom"]')!
      .getBoundingClientRect();
    return { frameTop: frameRect.top, frameBottom: frameRect.bottom, top: top.bottom, bottom: bottom.top };
  });
  expect(bars.frameTop).toBeGreaterThanOrEqual(bars.top);
  expect(bars.frameBottom).toBeLessThanOrEqual(bars.bottom);
});
