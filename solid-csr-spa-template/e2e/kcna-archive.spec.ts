import { mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { expect, test, type Page } from "@playwright/test";
import { installApiMocks, setUiPreferences, type AuthMode } from "./fixtures";
import { KCNA_DOCUMENT_ID, KCNA_ENGLISH_ID } from "./kcna-fixtures";

const ROOT = "/projects/kcna-archive";
const ENVELOPE_ERROR = (code: number, message: string) => ({
  success: false,
  error_code: code,
  message,
});

async function visit(page: Page, path: string, mode: AuthMode = "superuser", width = 1440): Promise<void> {
  await page.setViewportSize({ width, height: 900 });
  await installApiMocks(page, mode);
  await setUiPreferences(page, "en-US", "light");
  await page.goto(path);
}

async function expectNoHorizontalOverflow(page: Page): Promise<void> {
  const layout = await page.evaluate(() => ({
    document: document.documentElement.scrollWidth,
    viewport: document.documentElement.clientWidth,
  }));
  expect(layout.document).toBeLessThanOrEqual(layout.viewport + 1);
}

test.describe("access", () => {
  test("a superuser sees the Projects entry and the admin link, a visitor sees neither", async ({ page }) => {
    await visit(page, "/projects", "superuser");
    const entry = page.locator("a.kcna-entry");
    await expect(entry).toBeVisible();
    await expect(entry).toContainText("Administrators only");
    await entry.click();
    await expect(page).toHaveURL(new RegExp(`${ROOT}$`));
    await expect(page.getByRole("heading", { name: "KCNA Archive" })).toBeVisible();

    await page.goto("/admin/operations");
    await expect(page.locator(`.admin-workspace-sidebar a[href="${ROOT}"]`)).toBeVisible();
  });

  test("an authenticated non-superuser has no entry, no admin link, and is sent to 404", async ({ page }) => {
    await visit(page, "/projects", "authenticated");
    await expect(page.locator("a.kcna-entry")).toHaveCount(0);
    await expect(page.getByText("KCNA Archive")).toHaveCount(0);
    await page.goto(ROOT);
    await expect(page).toHaveURL(/\/404$/);
  });

  test("a signed-out visitor is sent to login and the palette lists no archive page", async ({ page }) => {
    await visit(page, "/projects", "logged-out");
    await expect(page.locator("a.kcna-entry")).toHaveCount(0);
    await page.keyboard.press("Control+k");
    await expect(page.getByRole("dialog")).toBeVisible();
    await expect(page.getByRole("dialog").getByText(/kcna/i)).toHaveCount(0);
    await page.keyboard.press("Escape");
    await page.goto(`${ROOT}/trends`);
    await expect(page).toHaveURL(/\/login/);
  });

  test("unknown archive sub-paths fall to the site 404", async ({ page }) => {
    await visit(page, `${ROOT}/nothing-here`);
    await expect(page.getByRole("heading", { name: "KCNA Archive" })).toHaveCount(0);
    await expect(page.locator(".kcna-tabs")).toHaveCount(0);
    await expect(page.locator(".not-found-stage, [class*='not-found']").first()).toBeVisible();
  });
});

test.describe("search", () => {
  test("results, highlights, filters, and shortcuts", async ({ page }) => {
    await visit(page, `${ROOT}?q=bridge`);
    await expect(page.locator(".kcna-row").first()).toBeVisible();
    await expect(page.locator(".kcna-pane-head h2")).toHaveText("4,210 results");
    await expect(page.locator(".kcna-row-title mark").first()).toHaveText(/bridge/i);
    await expect(page.locator(".kcna-row-title").nth(0)).toHaveAttribute("lang", "kor");
    await expect(page.locator(".kcna-row-title").nth(0)).toHaveAttribute("dir", "ltr");

    // A language filter is a link: it lands in the URL and keeps the text.
    await page.getByRole("button", { name: /^Korean/ }).click();
    await expect(page).toHaveURL(/lang=kor/);
    await expect(page).toHaveURL(/q=bridge/);

    // "/" focuses the field and selects its text; typing "j", "k", or "t" there is text.
    await page.keyboard.press("/");
    await expect(page.locator("#kcna-search-input")).toBeFocused();
    await expect(page.locator("#kcna-search-input")).toHaveValue("bridge");
    await page.keyboard.type("jkt");
    await expect(page.locator("#kcna-search-input")).toHaveValue("jkt");
    await expect(page).not.toHaveURL(/doc=|with=/);
    await page.locator("#kcna-search-input").fill("bridge");
    await page.locator("body").click({ position: { x: 4, y: 400 } });

    // "j" opens the first result beside the list; "k" never goes above it.
    await page.keyboard.press("j");
    await expect(page).toHaveURL(/doc=/);
    await expect(page.locator(".kcna-document")).toBeVisible();
    const first = await page.locator(".kcna-row[aria-current='true']").getAttribute("href");
    await page.keyboard.press("j");
    await expect(page.locator(".kcna-row[aria-current='true']")).not.toHaveAttribute("href", first ?? "");
    await page.keyboard.press("k");
    await expect(page.locator(".kcna-row[aria-current='true']")).toHaveAttribute("href", first ?? "");

    // "t" toggles the translation beside the document.
    await page.keyboard.press("t");
    await expect(page).toHaveURL(/with=eng/);
    await expect(page.locator(".kcna-parallel-row").first()).toBeVisible();
    await page.keyboard.press("t");
    await expect(page.locator(".kcna-parallel-row")).toHaveCount(0);
  });

  test("opening a result beside the list keeps the page where it was", async ({ page }) => {
    // A short window, so the list is taller than the page and the page scrolls.
    await page.setViewportSize({ width: 1440, height: 420 });
    await installApiMocks(page, "superuser");
    await setUiPreferences(page, "en-US", "light");
    await page.goto(`${ROOT}?q=bridge`);
    const row = page.locator(".kcna-row").nth(5);
    await row.scrollIntoViewIfNeeded();
    const before = await page.evaluate(() => window.scrollY);
    expect(before).toBeGreaterThan(100);
    await row.click();
    await expect(page).toHaveURL(/doc=/);
    await expect(page.locator(".kcna-document-title")).toBeVisible();
    // The click itself may nudge a row out from under the sticky header;
    // what must not happen is the jump back to the top of the page.
    expect(Math.abs((await page.evaluate(() => window.scrollY)) - before)).toBeLessThanOrEqual(40);

    // The keyboard and the links between language editions hold it too.
    await page.keyboard.press("j");
    await expect(page.locator(".kcna-row").nth(6)).toHaveAttribute("aria-current", "true");
    await page.locator(".kcna-translation-link").first().click();
    await expect(page).toHaveURL(new RegExp(`doc=${KCNA_ENGLISH_ID}`));
    expect(await page.evaluate(() => window.scrollY)).toBeGreaterThan(100);
  });

  test("a month histogram drag sets the date range in the URL", async ({ page }) => {
    await visit(page, ROOT);
    const chart = page.locator(".kcna-histogram-chart");
    await expect(chart).toBeVisible();
    const box = await chart.boundingBox();
    expect(box).not.toBeNull();
    if (box === null) return;
    await page.mouse.move(box.x + box.width * 0.2, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width * 0.4, box.y + box.height / 2, { steps: 4 });
    await page.mouse.up();
    await expect(page).toHaveURL(/from=\d{4}-\d{2}-01/);
    await expect(page).toHaveURL(/to=\d{4}-\d{2}-\d{2}/);
    await page.getByRole("button", { name: /^Clear range/ }).click();
    await expect(page).not.toHaveURL(/from=/);
  });

  test("the go-to-date form sets a month range", async ({ page }) => {
    await visit(page, ROOT);
    await page.getByPlaceholder("2026, 2026-11, or 2026-11-21").fill("2024-02");
    await page.getByRole("button", { name: "Go", exact: true }).click();
    await expect(page).toHaveURL(/from=2024-02-01/);
    await expect(page).toHaveURL(/to=2024-02-29/);
  });

  test("export links download from the export endpoint with the current query", async ({ page }) => {
    await visit(page, `${ROOT}?q=bridge&lang=eng`);
    const csv = page.getByRole("link", { name: "CSV" });
    await expect(csv).toHaveAttribute("href", /\/api\/admin\/kcna-archive\/export\?.*q=bridge.*lang=eng.*format=csv/);
    await expect(csv).toHaveAttribute("download", "");
  });

  test("a snapshot that is not ready shows a plain notice instead of an error", async ({ page }) => {
    await visit(page, ROOT);
    await page.route("**/api/admin/kcna-archive/search**", (route) =>
      route.fulfill({ status: 503, json: ENVELOPE_ERROR(113, "not installed") }),
    );
    await page.goto(ROOT);
    await expect(page.getByRole("status").filter({ hasText: "not installed on this server" })).toBeVisible();
    await expect(page.getByRole("alert")).toHaveCount(0);
  });

  test("a failed search offers a retry that works", async ({ page }) => {
    await visit(page, ROOT);
    let failing = true;
    await page.route("**/api/admin/kcna-archive/search**", async (route) => {
      if (failing) return route.fulfill({ status: 500, json: ENVELOPE_ERROR(118, "boom") });
      return route.fallback();
    });
    await page.goto(ROOT);
    await expect(page.getByRole("alert")).toContainText("The archive is not available right now.");
    failing = false;
    await page.getByRole("button", { name: "Retry" }).click();
    await expect(page.locator(".kcna-row").first()).toBeVisible();
  });

  test("an empty result says so", async ({ page }) => {
    await visit(page, ROOT);
    await page.route("**/api/admin/kcna-archive/search**", (route) =>
      route.fulfill({
        json: {
          success: true,
          data: { total: 0, offset: 0, limit: 50, items: [], facets: { languages: [], sources: [], categories: [] } },
          meta: {},
        },
      }),
    );
    await page.goto(`${ROOT}?q=nothing`);
    await expect(page.getByText("Nothing matches these filters.")).toBeVisible();
  });

  test("results open on their own page when the screen is narrow, without horizontal overflow", async ({ page }) => {
    await visit(page, `${ROOT}?q=bridge`, "superuser", 390);
    await expect(page.locator(".kcna-row").first()).toBeVisible();
    await expect(page.locator(".kcna-reader-pane")).toBeHidden();
    await expectNoHorizontalOverflow(page);
    await page.locator(".kcna-row").first().click();
    await expect(page).toHaveURL(/\/documents\//);
    await expect(page.locator(".kcna-document")).toBeVisible();
    await expect(page.getByRole("link", { name: /Back to results/ })).toHaveAttribute("href", /q=bridge/);
    await expectNoHorizontalOverflow(page);
  });
});

test.describe("reader", () => {
  test("side by side shows both editions and stacks on a narrow screen", async ({ page }) => {
    await visit(page, `${ROOT}/documents/${KCNA_DOCUMENT_ID}?q=bridge`);
    await expect(page.locator(".kcna-document-title")).toHaveAttribute("lang", "kor");
    await expect(page.getByRole("link", { name: "Source page" })).toHaveAttribute("rel", "noreferrer noopener");
    await expect(page.getByRole("link", { name: /Raw|stored/i })).toHaveCount(0);
    await page.getByRole("button", { name: "Side by side" }).first().click();
    await expect(page).toHaveURL(/with=eng/);
    const row = page.locator(".kcna-parallel-row").first();
    await expect(row).toBeVisible();
    const columns = await row.evaluate((element) => getComputedStyle(element).gridTemplateColumns.split(" ").length);
    expect(columns).toBe(2);

    await page.setViewportSize({ width: 390, height: 844 });
    const stacked = await row.evaluate((element) => getComputedStyle(element).gridTemplateColumns.split(" ").length);
    expect(stacked).toBe(1);
    await expectNoHorizontalOverflow(page);
  });

  test("a missing document is reported as not found", async ({ page }) => {
    await visit(page, ROOT);
    await page.route("**/api/admin/kcna-archive/documents/**", (route) =>
      route.fulfill({ status: 404, json: ENVELOPE_ERROR(112, "missing") }),
    );
    await page.goto(`${ROOT}/documents/${KCNA_ENGLISH_ID}`);
    await expect(page.getByRole("alert")).toContainText("Not found.");
  });
});

test.describe("trends and status", () => {
  test("terms are compared in the chart and table, and live in the URL", async ({ page }) => {
    await visit(page, `${ROOT}/trends`);
    await expect(page.getByText("Enter terms to compare.")).toBeVisible();
    await page.locator("#kcna-trends-input").fill("bridge, economy");
    await page.getByRole("button", { name: "Compare" }).click();
    await expect(page).toHaveURL(/terms=bridge%2Ceconomy/);
    await expect(page.locator(".kcna-chart-plot .kcna-line")).toHaveCount(2);
    await expect(page.locator(".kcna-table thead th")).toHaveCount(4);
    await page.getByRole("button", { name: "Month", exact: true }).click();
    await expect(page).toHaveURL(/bucket=month/);
    await expect(page.locator(".kcna-table tbody tr")).toHaveCount(36);
    await expectNoHorizontalOverflow(page);
  });

  test("status shows the snapshot and reloads it", async ({ page }) => {
    await visit(page, `${ROOT}/status`);
    await expect(page.getByText("Searchable articles")).toBeVisible();
    await expect(page.getByText("374,135")).toBeVisible();
    await page.getByRole("button", { name: "Reload snapshot" }).click();
    await expect(page.getByText("Snapshot reloaded.")).toBeVisible();
  });

  test("status explains each service state and a busy reload", async ({ page }) => {
    await visit(page, `${ROOT}/status`);
    await page.route("**/api/admin/kcna-archive/status", (route) =>
      route.fulfill({
        json: { success: true, data: { state: "not_installed", reloading: false, lastReloadFailed: false }, meta: {} },
      }),
    );
    await page.route("**/api/admin/kcna-archive/reload", (route) =>
      route.fulfill({ status: 409, json: ENVELOPE_ERROR(116, "busy") }),
    );
    await page.goto(`${ROOT}/status`);
    await expect(page.getByText("The archive snapshot is not installed on this server.")).toBeVisible();
    await page.getByRole("button", { name: "Reload snapshot" }).click();
    await expect(page.getByText("A reload is already running.")).toBeVisible();
  });
});

// Screenshots of every archive page for visual review. Set KCNA_RENDERS=1; the
// files land in target/kcna-archive-renders/.
const RENDER_LOCALES = ["en-US", "ko-KR", "ar", "de-DE"] as const;
const RENDER_WIDTHS = [390, 1440] as const;

test.describe("renders", () => {
  test.skip(process.env.KCNA_RENDERS === undefined, "set KCNA_RENDERS=1 to capture renders");
  for (const locale of RENDER_LOCALES) {
    for (const theme of ["light", "dark"] as const) {
      for (const width of RENDER_WIDTHS) {
        test(`render ${locale} ${theme} ${width}`, async ({ page }) => {
          test.setTimeout(90_000);
          await page.setViewportSize({ width, height: 900 });
          await installApiMocks(page, "superuser");
          await page.addInitScript(
            ({ tag, mode }) => {
              if (window !== window.top) return;
              localStorage.setItem("ui_locale", tag);
              localStorage.setItem("theme", mode);
              localStorage.setItem("motion", "off");
            },
            { tag: locale, mode: theme },
          );
          const directory = resolve(import.meta.dirname, "../../target/kcna-archive-renders");
          mkdirSync(directory, { recursive: true });
          const pages: ReadonlyArray<readonly [string, string]> = [
            ["search", `${ROOT}?q=bridge&doc=${KCNA_DOCUMENT_ID}&with=eng`],
            ["reader", `${ROOT}/documents/${KCNA_DOCUMENT_ID}?q=bridge&with=eng`],
            ["trends", `${ROOT}/trends?terms=bridge,economy,port,school`],
            ["status", `${ROOT}/status`],
            ["projects", "/projects"],
          ];
          for (const [name, path] of pages) {
            await page.goto(path, { waitUntil: "networkidle" });
            await page.evaluate(() => document.fonts.ready);
            await page.screenshot({
              path: resolve(directory, `${locale}-${theme}-${width}-${name}.png`),
              fullPage: true,
              animations: "disabled",
            });
            await expectNoHorizontalOverflow(page);
          }
        });
      }
    }
  }
});
