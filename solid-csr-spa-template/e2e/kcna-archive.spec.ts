import { mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { expect, test, type Page } from "@playwright/test";
import { installApiMocks, setUiPreferences, type AuthMode } from "./fixtures";
import {
  archiveAccessBody,
  archiveReader,
  archiveReadersBody,
  KCNA_DOCUMENT_ID,
  KCNA_ENGLISH_ID,
  type ArchiveReaderFixture,
} from "./kcna-fixtures";

const ROOT = "/projects/kcna-archive";
const ADMIN = "/admin/kcna-archive";
const DISCLAIMER = /ROK Army reservist/;

/** The signed-in member is on the reader list. */
async function listMember(page: Page): Promise<void> {
  await page.route("**/api/kcna-archive/access", (route) =>
    route.fulfill({ json: archiveAccessBody(true, false) }),
  );
}

const ALICE = "0198f4d0-bbbb-7000-8000-000000000001";
const BORIS = "0198f4d0-bbbb-7000-8000-000000000002";

/**
 * A reader list that remembers its changes, and a member directory with two
 * accounts, so the access page can be driven as against a server. Returns the
 * requests the page made to change the list.
 */
async function serveReaderList(page: Page): Promise<string[]> {
  let readers: ArchiveReaderFixture[] = [];
  const changes: string[] = [];
  await page.route("**/api/admin/authorization/users**", (route) => {
    const search = new URL(route.request().url()).searchParams.get("search") ?? "";
    const users = [
      { user_id: ALICE, user_name: "Alice", user_email: "alice@example.test", role_id: ALICE, role_name: "user" },
      { user_id: BORIS, user_name: "Boris", user_email: "boris@example.test", role_id: ALICE, role_name: "user" },
    ].filter((user) => user.user_name.toLowerCase().startsWith(search.toLowerCase()));
    return route.fulfill({ json: { success: true, data: { users, next_cursor: null }, meta: {} } });
  });
  await page.route("**/api/admin/kcna-archive/readers**", (route) => {
    const request = route.request();
    if (request.method() === "POST") {
      const { userId } = request.postDataJSON() as { userId: string };
      changes.push(`POST ${JSON.stringify(request.postDataJSON())}`);
      readers = [archiveReader(userId, userId === ALICE ? "Alice" : "Boris"), ...readers];
    } else if (request.method() === "DELETE") {
      const userId = new URL(request.url()).pathname.split("/").pop() ?? "";
      changes.push(`DELETE ${userId}`);
      readers = readers.filter((reader) => reader.userId !== userId);
    }
    return route.fulfill({ json: archiveReadersBody(readers) });
  });
  await page.route("**/api/kcna-archive/access", (route) =>
    route.fulfill({ json: archiveAccessBody(true, readers.length === 0) }),
  );
  return changes;
}

async function expectNotFoundInPlace(page: Page, path: string): Promise<void> {
  await page.goto(path);
  await expect(page.locator(".not-found-stage")).toBeVisible();
  await expect(page.locator(".kcna-tabs")).toHaveCount(0);
  // The page is answered where it was asked for: no sign-in page, no redirect.
  expect(new URL(page.url()).pathname).toBe(path);
}
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
  test("a superuser reaches the archive from Projects and its access page from the admin workspace", async ({ page }) => {
    await visit(page, "/projects", "superuser");
    const entry = page.locator("a.kcna-entry");
    await expect(entry).toBeVisible();
    await expect(entry).toContainText("Administrators only");

    // The Projects menu in the top bar lists the archive too.
    await page.getByRole("button", { name: "Projects" }).click();
    const menuLink = page.locator(`.site-nav-panel a[href="${ROOT}"]`);
    await expect(menuLink).toHaveText("KCNA Archive");
    await menuLink.click();
    await expect(page).toHaveURL(new RegExp(`${ROOT}$`));
    await expect(page.getByRole("heading", { name: "KCNA Archive" })).toBeVisible();
    await expect(page.locator(".kcna-header .kcna-badge")).toHaveText("Administrators only");
    await expect(page.locator(".kcna-disclaimer")).toHaveText(DISCLAIMER);
    await expect(page.locator(".kcna-tab")).toHaveText(["Search", "Trends", "Status"]);

    await page.goto("/admin/operations");
    const sidebar = page.locator(".admin-workspace-sidebar");
    await expect(sidebar.locator(`a[href="${ADMIN}"]`)).toBeVisible();
    await expect(sidebar.locator(`a[href="${ROOT}"]`)).toHaveCount(0);
  });

  test("a member who is not listed sees nothing of the archive and gets Not Found for it", async ({ page }) => {
    await visit(page, "/projects", "authenticated");
    await expect(page.locator("a.kcna-entry")).toHaveCount(0);
    await expect(page.getByText("KCNA Archive")).toHaveCount(0);
    await page.getByRole("button", { name: "Projects" }).click();
    await expect(page.locator(".site-nav-panel a")).not.toHaveCount(0);
    await expect(page.locator(`.site-nav-panel a[href="${ROOT}"]`)).toHaveCount(0);
    await expectNotFoundInPlace(page, ROOT);
    await expectNotFoundInPlace(page, `${ROOT}/status`);
  });

  test("a listed member reads the archive but cannot manage it", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "authenticated");
    await listMember(page);
    await setUiPreferences(page, "en-US", "light");
    await page.goto("/projects");
    const entry = page.locator("a.kcna-entry");
    await expect(entry).toBeVisible();
    await expect(entry.locator(".kcna-badge")).toHaveCount(0);
    await entry.click();
    await expect(page.locator(".kcna-row").first()).toBeVisible();
    await expect(page.locator(".kcna-header .kcna-badge")).toHaveCount(0);
    await expect(page.locator(".kcna-disclaimer")).toHaveText(DISCLAIMER);
    await expect(page.locator(".kcna-tab")).toHaveText(["Search", "Trends"]);
    await page.goto(`${ROOT}/trends?terms=bridge`);
    await expect(page.locator(".kcna-table")).toBeVisible();
    await expectNotFoundInPlace(page, `${ROOT}/status`);
  });

  test("a signed-out visitor gets Not Found, not the sign-in page, and the palette lists no archive", async ({ page }) => {
    await visit(page, "/projects", "logged-out");
    await expect(page.locator("a.kcna-entry")).toHaveCount(0);
    await page.keyboard.press("Control+k");
    await expect(page.getByRole("dialog")).toBeVisible();
    await expect(page.getByRole("dialog").getByText(/kcna/i)).toHaveCount(0);
    await page.keyboard.press("Escape");
    for (const path of [ROOT, `${ROOT}/trends`, `${ROOT}/documents/${KCNA_DOCUMENT_ID}`, `${ROOT}/status`]) {
      await expectNotFoundInPlace(page, path);
    }
  });

  test("unknown archive sub-paths fall to the site 404", async ({ page }) => {
    await visit(page, `${ROOT}/nothing-here`);
    await expect(page.getByRole("heading", { name: "KCNA Archive" })).toHaveCount(0);
    await expect(page.locator(".kcna-tabs")).toHaveCount(0);
    await expect(page.locator(".not-found-stage, [class*='not-found']").first()).toBeVisible();
  });

  test("a superuser allows a member from the access page and removes them again", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "superuser");
    const changes = await serveReaderList(page);
    await setUiPreferences(page, "en-US", "light");
    await page.goto(ADMIN);
    const readers = page.locator('[data-list="readers"]');
    await expect(readers).toContainText("Nobody is listed. Only administrators can read the archive.");

    await page.getByLabel("Find a member by name or email").fill("a");
    await page.locator(".kcna-access-search").getByRole("button", { name: "Search" }).click();
    const matches = page.locator('[data-list="matches"] .kcna-access-row');
    await expect(matches).toHaveCount(1);
    await expect(matches).toContainText("alice@example.test");
    await matches.getByRole("button", { name: "Allow" }).click();
    await expect(page.getByRole("status").filter({ hasText: "Access list updated." })).toBeVisible();
    await expect(readers.locator(".kcna-access-row")).toHaveCount(1);
    await expect(readers).toContainText("Alice");
    // An account that is already listed cannot be allowed twice.
    await expect(matches.getByRole("button", { name: "Allow" })).toBeDisabled();
    expect(changes).toEqual([`POST {"userId":"${ALICE}"}`]);

    // The "administrators only" badge is gone once somebody else is listed.
    await page.locator(`.kcna-access-head a[href="${ROOT}"]`).click();
    await expect(page.locator(".kcna-row").first()).toBeVisible();
    await expect(page.locator(".kcna-header .kcna-badge")).toHaveCount(0);

    await page.goto(ADMIN);
    await expect(readers.locator(".kcna-access-row")).toHaveCount(1);
    await readers.getByRole("button", { name: "Remove" }).click();
    await expect(readers).toContainText("Nobody is listed.");
    expect(changes).toEqual([`POST {"userId":"${ALICE}"}`, `DELETE ${ALICE}`]);
  });

  test("a search without a match says so, and a failed change is reported", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "superuser");
    await serveReaderList(page);
    await setUiPreferences(page, "en-US", "light");
    await page.goto(ADMIN);
    const field = page.getByLabel("Find a member by name or email");
    await field.fill("zzz");
    await field.press("Enter");
    await expect(page.locator('[data-list="matches"]')).toContainText("No member matches.");

    await page.route("**/api/admin/kcna-archive/readers", async (route) => {
      if (route.request().method() !== "POST") return route.fallback();
      return route.fulfill({ status: 500, json: ENVELOPE_ERROR(120, "store failed") });
    });
    await field.fill("b");
    await field.press("Enter");
    await page.locator('[data-list="matches"]').getByRole("button", { name: "Allow" }).click();
    await expect(page.getByRole("alert").filter({ hasText: "The access list could not be changed." })).toBeVisible();
    await expect(page.locator('[data-list="readers"]')).toContainText("Nobody is listed.");
  });

  test("a non-superuser cannot open the access page", async ({ page }) => {
    await visit(page, ADMIN, "authenticated");
    await expect(page.locator(".kcna-access")).toHaveCount(0);
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
    await expect(csv).toHaveAttribute("href", /\/api\/kcna-archive\/export\?.*q=bridge.*lang=eng.*format=csv/);
    await expect(csv).toHaveAttribute("download", "");
  });

  test("a snapshot that is not ready shows a plain notice instead of an error", async ({ page }) => {
    await visit(page, ROOT);
    await page.route("**/api/kcna-archive/search**", (route) =>
      route.fulfill({ status: 503, json: ENVELOPE_ERROR(113, "not installed") }),
    );
    await page.goto(ROOT);
    await expect(page.getByRole("status").filter({ hasText: "not installed on this server" })).toBeVisible();
    await expect(page.getByRole("alert")).toHaveCount(0);
  });

  test("a failed search offers a retry that works", async ({ page }) => {
    await visit(page, ROOT);
    let failing = true;
    await page.route("**/api/kcna-archive/search**", async (route) => {
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
    await page.route("**/api/kcna-archive/search**", (route) =>
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
    await page.route("**/api/kcna-archive/documents/**", (route) =>
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
            ["access", ADMIN],
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
