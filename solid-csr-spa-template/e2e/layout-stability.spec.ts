import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, test, type Page } from "@playwright/test";
import { installApiMocks, type AuthMode } from "./fixtures";
import { UI_LOCALES } from "../src/i18n/locales";

/** Controls keep one size in every language; only running text may reflow. */
const ROUTES: ReadonlyArray<readonly [string, AuthMode]> = [
  ["/", "superuser"],
  ["/", "logged-out"],
  ["/about", "logged-out"],
  ["/about/experience", "logged-out"],
  ["/about-blog", "logged-out"],
  ["/about-blog/performance", "logged-out"],
  ["/blog", "superuser"],
  ["/forum", "superuser"],
  ["/forum/new", "superuser"],
  ["/forum/notifications", "superuser"],
  ["/live-chat", "superuser"],
  ["/visitor-board", "logged-out"],
  ["/photographs", "superuser"],
  ["/projects", "superuser"],
  ["/geo-ip-db", "logged-out"],
  ["/eu5-locations-db", "logged-out"],
  ["/backend-stats", "logged-out"],
  ["/login", "logged-out"],
  ["/register", "logged-out"],
  ["/find-password", "logged-out"],
  ["/edit-profile", "superuser"],
  ["/admin/operations", "superuser"],
  ["/admin/authorization", "superuser"],
  ["/404", "logged-out"],
];
// Narrow a local run with LAYOUT_WIDTHS=390,1440 or LAYOUT_LOCALES=de-DE,ar.
const WIDTHS = (process.env.LAYOUT_WIDTHS ?? "390,1024,1440,1920").split(",").map(Number);
const LOCALES = UI_LOCALES.filter(({ tag }) => !process.env.LAYOUT_LOCALES || process.env.LAYOUT_LOCALES.split(",").includes(tag));
const TOLERANCE_PX = 0.75;

type Box = { key: string; css: string; width: number; height: number; start: number; top: number; clipped: boolean; bar: boolean; inner: boolean; label: string };

async function measure(page: Page): Promise<Box[]> {
  return page.evaluate(() => {
    const CONTROLS = [
      "button", "select", "input:not([type=hidden])", "textarea", "summary", "th",
      "[role=tab]", "[role=menuitem]", "[role=button]", ".ui-button", "nav a",
      "[data-site-bar] a", "[data-site-bar] li", "[data-site-bar] nav", "[data-site-bar] > div",
      ".site-header-row", ".site-header-brand", ".site-header-actions", ".site-status-layout > *",
      ".site-nav-label", ".site-status-layout > * > *",
    ].join(",");
    const path = (element: Element): string => {
      const parts: string[] = [];
      for (let node: Element | null = element; node && node !== document.body; node = node.parentElement) {
        const parent: Element | null = node.parentElement;
        const index = parent ? [...parent.children].indexOf(node) : 0;
        parts.unshift(`${node.tagName.toLowerCase()}[${index}]`);
      }
      return parts.join(">");
    };
    const viewport = document.documentElement.clientWidth;
    const rtl = document.documentElement.dir === "rtl";
    return [...document.querySelectorAll(CONTROLS)].flatMap((element) => {
      const box = element.getBoundingClientRect();
      const style = getComputedStyle(element);
      if (box.width === 0 || box.height === 0 || style.visibility === "hidden") return [];
      const bar = element.closest("[data-site-bar]") !== null;
      const inner = element.matches(".site-nav-label, .site-status-layout > * > *");
      return [{
        key: path(element),
        css: String(element.getAttribute("class") ?? "").split(/\s+/).filter((name) => /^[a-z]+-[a-z-]+$|^ui-/.test(name) && !/^(text|bg|border|font|rounded|items|justify|hover|transition|duration|inline|leading|tracking|flex|min|max|gap|shadow|placeholder|underline|decoration|no|select|cursor|sm|md|lg)-/.test(name)).slice(0, 3).join("."),
        width: box.width,
        height: box.height,
        // Mirrored so a right-to-left page compares against the same reading position.
        start: rtl ? viewport - box.right : box.left,
        top: box.top,
        // A cut label is only acceptable when the full text is one hover away.
        clipped: element.scrollWidth > element.clientWidth + 1 && style.overflowX !== "visible"
          && !element.hasAttribute("title") && !element.closest(".site-status-layout"),
        bar,
        inner,
        label: (element.getAttribute("aria-label") ?? element.textContent ?? "").trim().slice(0, 32),
      }];
    });
  });
}

async function load(page: Page, route: string, tag: string): Promise<Box[]> {
  await page.evaluate((language) => localStorage.setItem("ui_locale", language), tag);
  await page.goto(route);
  await expect(page.locator("html")).toHaveAttribute("lang", tag);
  await page.locator(".site-header-brand").waitFor();
  await page.waitForLoadState("load");
  await page.waitForTimeout(350);
  await page.evaluate(() => document.fonts.ready);
  return measure(page);
}

for (const width of WIDTHS) {
  for (const [route, mode] of ROUTES) {
    test(`${route} (${mode}) keeps control sizes in every language at ${width}px`, async ({ page }) => {
      test.setTimeout(30_000 + LOCALES.length * 3_000);
      await page.setViewportSize({ width, height: 900 });
      await installApiMocks(page, mode);
      await page.route("**/api/i18n/ui-text*", (intercepted) => intercepted.abort());
      await page.addInitScript(() => localStorage.setItem("motion", "off"));
      await page.goto("/404");
      const baseline = await load(page, route, "en-US");
      const byKey = new Map(baseline.map((box) => [box.key, box]));
      const offenders: string[] = [];
      for (const { tag } of LOCALES) {
        const boxes = tag === "en-US" ? baseline : await load(page, route, tag);
        for (const box of boxes) {
          const reference = byKey.get(box.key);
          if (box.clipped) offenders.push(`${tag} clipped ${box.key} .${box.css} "${box.label}"`);
          // A label inside a fixed cell may be any width as long as it is not cut.
          if (!reference || box.inner) continue;
          const widthDelta = Math.abs(box.width - reference.width);
          const heightDelta = Math.abs(box.height - reference.height);
          // Bars are fixed chrome, so their positions must hold as well.
          // The English copyright sentence stays left to right, so its link cannot mirror.
          const moved = box.bar && !box.key.includes("span[1]>a[0]") && (Math.abs(box.start - reference.start) > TOLERANCE_PX || Math.abs(box.top - reference.top) > TOLERANCE_PX);
          if (widthDelta > TOLERANCE_PX || heightDelta > TOLERANCE_PX || moved) {
            offenders.push(`${tag} ${box.key} .${box.css} "${reference.label}" -> "${box.label}" w ${reference.width.toFixed(1)}->${box.width.toFixed(1)} h ${reference.height.toFixed(1)}->${box.height.toFixed(1)}-> top ->` : ""}`);
          }
        }
      }
      const directory = resolve("../target/layout-stability");
      mkdirSync(directory, { recursive: true });
      writeFileSync(
        resolve(directory, `${route.replaceAll("/", "_") || "_"}-${mode}-${width}.json`),
        JSON.stringify(offenders, null, 1),
      );
      expect(offenders).toEqual([]);
    });
  }
}
