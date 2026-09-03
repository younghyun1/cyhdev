import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";
import { installApiMocks, setUiPreferences } from "./fixtures";

const viewports = [
  { name: "narrow portrait", width: 320, height: 568 },
  { name: "regular portrait", width: 390, height: 844 },
  { name: "landscape", width: 844, height: 390 },
] as const;

const routeFamilies = [
  "/",
  "/about",
  "/login",
  "/blog",
  "/blog/11111111-1111-4111-8111-111111111111",
  "/forum",
  "/live-chat",
  "/photographs",
  "/projects",
  "/visitor-board",
  "/backend-stats",
  "/admin/authorization",
  "/admin/operations",
  "/eu5-locations-db",
  "/404",
] as const;

const everySolidRoute = [
  ...routeFamilies,
  "/about-blog",
  "/find-password",
  "/reset-password",
  "/verify-email",
  "/blog/new",
  "/blog/11111111-1111-4111-8111-111111111111/edit",
  "/forum/new",
  "/forum/notifications",
  "/forum/11111111-1111-4111-8111-111111111111",
  "/users/mobile-superuser",
  "/geo-ip-db",
  "/register",
  "/edit-profile",
  "/under-construction",
] as const;

async function expectMobileReflow(page: Page): Promise<void> {
  await expect(page.locator("[data-site-bar='top']")).toBeVisible();
  await page.waitForTimeout(20);
  const layout = await page.evaluate(() => {
    const visibleOverflow = window.innerWidth >= 768 ? [] : Array.from(
      document.querySelectorAll<HTMLElement>(
        "button, a[href], input, select, textarea, [role='button']",
      ),
    )
      .filter((element) => {
        const style = getComputedStyle(element);
        const rect = element.getBoundingClientRect();
        return (
          style.visibility !== "hidden" &&
          style.display !== "none" &&
          rect.width > 0 &&
          rect.height > 0
        );
      })
      .filter((element) => {
        const rect = element.getBoundingClientRect();
        if (rect.left >= -1 && rect.right <= window.innerWidth + 1) {
          return false;
        }
        let ancestor = element.parentElement;
        while (ancestor) {
          const overflow = getComputedStyle(ancestor).overflowX;
          if (
            (overflow === "auto" || overflow === "scroll") &&
            ancestor.scrollWidth > ancestor.clientWidth
          ) {
            return false;
          }
          ancestor = ancestor.parentElement;
        }
        return true;
      })
      .map((element) => element.outerHTML.slice(0, 120));
    return {
      documentWidth: document.documentElement.scrollWidth,
      viewportWidth: document.documentElement.clientWidth,
      visibleOverflow,
    };
  });
  expect(layout.documentWidth).toBeLessThanOrEqual(layout.viewportWidth + 1);
  expect(layout.visibleOverflow).toEqual([]);
}

async function expectFullViewportHeight(
  page: Page,
  selector: string,
): Promise<void> {
  const height = await page.locator(selector).evaluate((element) =>
    element.getBoundingClientRect().height,
  );
  expect(Math.abs(height - (page.viewportSize()?.height ?? 0))).toBeLessThan(1);
}

for (const viewport of viewports) {
  for (const locale of ["en-US", "ko-KR"] as const) {
    for (const theme of ["light", "dark"] as const) {
      test(`${viewport.name} reflows route families in ${locale} ${theme}`, async ({
        page,
      }) => {
        await page.setViewportSize(viewport);
        await installApiMocks(page, "superuser");
        await setUiPreferences(page, locale, theme);
        for (const path of routeFamilies) {
          await page.goto(path, { waitUntil: "domcontentloaded" });
          await expectMobileReflow(page);
        }
      });
    }
  }
}

test("every Solid route reflows at 320 CSS pixels", async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 568 });
  await installApiMocks(page, "superuser");
  await setUiPreferences(page, "en-US", "light");
  for (const path of everySolidRoute) {
    await page.goto(path, { waitUntil: "domcontentloaded" });
    await expectMobileReflow(page);
  }
});

test("drawer contains settings and auth actions, traps focus, and restores it", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installApiMocks(page, "authenticated");
  await page.goto("/");
  const trigger = page.getByRole("button", {
    name: /navigation|메뉴|sidebar/i,
  });
  await trigger.click();
  const drawer = page.getByRole("dialog");
  await expect(drawer).toBeVisible();
  await expect(drawer.getByLabel(/language|언어/i)).toBeVisible();
  await expect(
    drawer.getByRole("link", { name: /edit profile|프로필/i }),
  ).toBeVisible();
  await expect(page.locator("#app-root")).toHaveAttribute("inert", "");
  await page.keyboard.press("Escape");
  await expect(drawer).toBeHidden();
  await expect(trigger).toBeFocused();
});

test("logged-out drawer exposes login and shared system status", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 568 });
  await installApiMocks(page, "logged-out");
  await page.goto("/");
  await page
    .getByRole("button", { name: /navigation|메뉴|sidebar/i })
    .click();
  const drawer = page.getByRole("dialog");
  await expect(drawer.getByRole("link", { name: "Login" })).toBeVisible();
  await expect(drawer.getByText("Site status")).toBeVisible();
});

test("form focus hides the mobile status target", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installApiMocks(page, "logged-out");
  await page.goto("/login");
  await page.getByPlaceholder("Email").focus();
  await expect(page.locator("[data-site-bar='bottom']")).toBeHidden();
});

test("mobile dialogs fill the visual viewport and photo details swipe", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installApiMocks(page, "superuser");
  await page.goto("/photographs");
  await page.locator(".photo-card").first().click();
  const detail = page.locator(".details-modal");
  await expect(detail).toBeVisible();
  await expectFullViewportHeight(page, ".details-modal");
  await page.locator(".details-image-container").evaluate((element) => {
    const start = new Event("touchstart", { bubbles: true });
    Object.defineProperty(start, "touches", {
      value: [{ clientX: 300, clientY: 200 }],
    });
    element.dispatchEvent(start);
    const end = new Event("touchend", { bubbles: true });
    Object.defineProperty(end, "changedTouches", {
      value: [{ clientX: 100, clientY: 205 }],
    });
    element.dispatchEvent(end);
  });
  await expect(page).toHaveURL(/22222222-2222-4222-8222-222222222222/);
  await page.keyboard.press("Escape");
  await expect(detail).toBeHidden();

  const upload = page.getByRole("button", { name: /upload photo/i });
  await upload.click();
  const uploadDialog = page.getByRole("dialog", {
    name: /upload photographs/i,
  });
  await expect(uploadDialog).toBeVisible();
  await expectFullViewportHeight(page, ".upload-modal");
  await page.keyboard.press("Escape");
  await expect(upload).toBeFocused();
});

test("deep comments and prose stay contained", async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 568 });
  await installApiMocks(page, "superuser");
  await page.goto("/blog/11111111-1111-4111-8111-111111111111");
  await expectMobileReflow(page);
  const margins = await page
    .locator(".threaded-comment")
    .evaluateAll((items) =>
      items.map((item) => Number.parseFloat(getComputedStyle(item).marginLeft)),
    );
  expect(Math.max(...margins)).toBeLessThanOrEqual(36);
  await expect(page.locator(".blog-vote-rail")).toHaveCSS(
    "flex-direction",
    "row",
  );
});

test("project dialog offers a separate mobile launch target", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installApiMocks(page, "superuser");
  await page.goto("/projects");
  await page.locator(".project-card").click();
  await expect(
    page.getByRole("link", { name: "Open separately" }),
  ).toBeVisible();
  await expectFullViewportHeight(page, ".wasm-modal");
});

test("mobile controls meet the declared target floors", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installApiMocks(page, "superuser");
  await page.goto("/admin/authorization");
  const sizes = await page
    .locator("button:visible, [role='button']:visible")
    .evaluateAll((items) =>
      items.map((item) => {
        const rect = item.getBoundingClientRect();
        return {
          primary: item.classList.contains("ui-button"),
          width: rect.width,
          height: rect.height,
        };
      }),
    );
  for (const size of sizes) {
    expect(size.width).toBeGreaterThanOrEqual(24);
    expect(size.height).toBeGreaterThanOrEqual(size.primary ? 44 : 24);
  }
});

test("core mobile views have no detectable WCAG A or AA violations", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installApiMocks(page, "superuser");
  for (const path of [
    "/",
    "/login",
    "/forum",
    "/admin/authorization",
  ] as const) {
    await page.goto(path);
    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
      .analyze();
    expect(results.violations).toEqual([]);
  }
});
