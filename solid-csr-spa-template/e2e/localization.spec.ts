import { expect, test } from "@playwright/test";
import { installApiMocks } from "./fixtures";
import { UI_LOCALES } from "../src/i18n/locales";
import { resolve } from "node:path";

const loginLabels = {
  "en-US": "Login", "ko-KR": "로그인", "fr-FR": "Connexion",
  "es-ES": "Iniciar sesión", "zh-Hans": "登录", "zh-Hant": "登入",
  "ja-JP": "ログイン", "de-DE": "Anmelden",
};

test("Korean Home introduction and About pages translate", async ({ page }) => {
  await installApiMocks(page, "logged-out");
  await page.addInitScript(() => {
    if (!localStorage.getItem("ui_locale")) localStorage.setItem("ui_locale", "en-US");
  });
  await page.route("**/api/i18n/ui-text*", (route) => route.abort());
  await page.goto("/");
  await page.locator("select").first().selectOption("ko-KR");
  const introduction = page.locator(".home-hero-inner");
  await expect(introduction).toContainText("소프트웨어 엔지니어");
  await expect(introduction).toContainText("Soundpatrol에서 사이버 보안, DevOps, 웹 서비스 및 개발자 도구 관련 업무를 하고 있습니다.");
  await expect(introduction).toContainText("직접 서버를 운영하고, 사진을 찍고, Rust로 도구를 만드는 것도 좋아합니다.");
  await expect(introduction).toContainText("장인 정신을 가지고 작업하는 것을 지향합니다.");
  await expect(introduction).not.toContainText("I work on cybersecurity");
  await page.reload();
  await expect(introduction).toContainText("장인 정신을 가지고 작업하는 것을 지향합니다.");
  for (const [route, heading] of [["/about", "소개"], ["/about-blog", "블로그 기술 구성"]]) {
    await page.goto(route);
    await expect(page.locator("html")).toHaveAttribute("lang", "ko-KR");
    await expect(page.locator('main[lang="ko-KR"]')).toBeVisible();
    await expect(page.getByRole("heading", { name: heading, exact: true })).toBeVisible();
  }
});

for (const { tag } of UI_LOCALES) {
  test(`${tag} loads offline fallback, survives reload, and fits mobile`, async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await installApiMocks(page, "logged-out");
    await page.addInitScript((language) => localStorage.setItem("ui_locale", language), tag);
    await page.route("**/api/i18n/ui-text*", (route) => route.abort());
    await page.goto("/login");
    await expect(page.locator("html")).toHaveAttribute("lang", tag);
    expect(await page.locator("select").first().locator("option").allTextContents()).toEqual([
      "English", "한국어", "Français", "Español", "简体中文", "繁體中文", "日本語", "Deutsch",
    ]);
    await expect(page.getByRole("heading", { name: loginLabels[tag], exact: true })).toBeVisible();
    await expect(page).toHaveTitle("Young Hyun Chi | Software Engineer");
    await expect(page.locator(".site-header-brand span").last()).toHaveText("Younghyun's Blog");
    await page.reload();
    await expect(page.getByRole("heading", { name: loginLabels[tag], exact: true })).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: resolve("../target/localization-renders", `${tag}-login-mobile.png`), fullPage: true });
  });
}

test("switching languages discards a late response and persists selection", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await installApiMocks(page, "logged-out");
  await page.goto("/login");
  let releaseResponse: (() => void) | undefined;
  await page.route("**/api/i18n/ui-text*", async (route) => {
    if (new URL(route.request().url()).searchParams.get("locale") === "fr-FR") {
      await new Promise<void>((resolve) => { releaseResponse = resolve; });
      await route.fulfill({ json: { success: true, data: { locale: "fr-FR", texts: { "page.login.title": "Stale response" } } } });
    } else {
      await route.abort();
    }
  });
  const select = page.locator("select").first();
  await select.selectOption("fr-FR");
  await expect.poll(() => typeof releaseResponse).toBe("function");
  await select.selectOption("ja-JP");
  await expect(page.getByRole("heading", { name: "ログイン", exact: true })).toBeVisible();
  const staleResponse = page.waitForResponse((response) => response.url().includes("locale=fr-FR"));
  releaseResponse?.();
  await staleResponse;
  await expect(page.locator("html")).toHaveAttribute("lang", "ja-JP");
  await expect(page.getByText("Stale response")).toHaveCount(0);
  expect(await page.evaluate(() => localStorage.getItem("ui_locale"))).toBe("ja-JP");
  await page.reload();
  await expect(page.getByRole("heading", { name: "ログイン", exact: true })).toBeVisible();
});

for (const width of [768, 1024, 1280, 1440, 1920]) {
  test(`translated authenticated desktop header does not overlap at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    await installApiMocks(page, "superuser");
    await page.addInitScript(() => localStorage.setItem("ui_locale", "de-DE"));
    await page.goto("/");
    await expect(page.locator("html")).toHaveAttribute("lang", "de-DE");
    await expect(page.locator(".site-header-identity")).toBeVisible();
    await expect(page.locator(".site-header-navigation")).toContainText("Startseite");
    await page.evaluate(() => document.fonts.ready);
    const bounds = await page.evaluate(() => {
      const rect = (selector: string) => {
        const element = document.querySelector(selector);
        if (!element) throw new Error(`Missing ${selector}`);
        const { left, right, top, bottom } = element.getBoundingClientRect();
        return { left, right, top, bottom };
      };
      return {
        brand: rect(".site-header-brand"), nav: rect(".site-header-navigation"),
        actions: rect(".site-header-actions"),
        horizontalOverflow: document.documentElement.scrollWidth > innerWidth,
      };
    });
    expect(bounds.horizontalOverflow).toBe(false);
    expect(bounds.brand.right).toBeLessThanOrEqual(bounds.actions.left);
    const separateRows = bounds.nav.top >= bounds.actions.bottom || bounds.nav.bottom <= bounds.actions.top;
    expect(separateRows || bounds.nav.right <= bounds.actions.left).toBe(true);
    await page.screenshot({ path: resolve("../target/localization-renders", `de-DE-header-${width}.png`) });
  });
}
