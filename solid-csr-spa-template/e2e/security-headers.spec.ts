import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, test, type Frame, type Page } from "@playwright/test";
import { installApiMocks, setUiPreferences } from "./fixtures";

// The backend unit test `browser_check_fixture_matches_the_backend_policy` keeps this
// file identical to the headers the server sends for the preview origin.
type SecurityHeaders = {
  readonly contentSecurityPolicy: string;
  readonly permissionsPolicy: string;
  readonly mapPolicy: string;
  readonly eu5Policy: string;
};

const here = import.meta.dirname;
const policy = JSON.parse(
  readFileSync(resolve(here, "security-headers.json"), "utf8"),
) as SecurityHeaders;
const storageShim = readFileSync(
  resolve(here, "../../rust-be-template/src/routers/main_router/squaremap_storage_shim.js"),
  "utf8",
);
const eu5HostPath = resolve(here, "../../vendor/eu5-location-filter/web/index.html");
const id = "11111111-1111-4111-8111-111111111111";
const map = readFileSync(resolve(here, "../../docs/design/fe/site-map.md"), "utf8");
const routes = [...map.matchAll(/^\| `([^`]+)` \| (Public|Signed in|Superuser|Fallback) \|/gm)]
  .map((match) => match[1] ?? "/404")
  .map((pattern) => pattern.replace(/:post_id|:topic_id|:photograph_id/g, id)
    .replace(":userName", "mobile-superuser").replace("*404", "/missing-page-fixture"));

type Violation = { directive: string; blocked: string; sample: string };

/** Serves every main-frame document with the backend's application headers. */
async function serveWithApplicationPolicy(page: Page): Promise<void> {
  await page.route("**/*", async (route) => {
    const request = route.request();
    if (request.resourceType() !== "document" || request.frame() !== page.mainFrame()) {
      await route.fallback();
      return;
    }
    const response = await route.fetch();
    await route.fulfill({
      response,
      headers: {
        ...response.headers(),
        "content-security-policy": policy.contentSecurityPolicy,
        "permissions-policy": policy.permissionsPolicy,
      },
    });
  });
}

/** Stands in for squaremap: its layer control needs localStorage in the sandbox. */
async function serveSandboxedMap(page: Page): Promise<void> {
  await page.route("**/minecraft/map/", (route) => route.fulfill({
    contentType: "text/html",
    headers: { "content-security-policy": policy.mapPolicy, "access-control-allow-origin": "*" },
    body: `<!doctype html><html><head><script>${storageShim}</script></head><body><h1>World map</h1>`
      + `<script>localStorage.setItem("hide_players", "true");`
      + `document.body.dataset.probe = localStorage.getItem("hide_players") + " " + self.origin;</script></body></html>`,
  }));
}

/** Serves the real EU5 host document with a stub for its generated WebAssembly module. */
async function serveSandboxedEu5(page: Page): Promise<void> {
  const host = readFileSync(eu5HostPath, "utf8");
  await page.route("**/eu5-locations-db/app/index.html", (route) => route.fulfill({
    contentType: "text/html",
    headers: { "content-security-policy": policy.eu5Policy, "access-control-allow-origin": "*" },
    body: host,
  }));
  // Module scripts from an opaque origin are CORS requests; without the header they fail.
  await page.route("**/eu5-locations-db/app/pkg/eu5_location_filter.js", (route) => route.fulfill({
    contentType: "text/javascript",
    headers: { "access-control-allow-origin": "*" },
    body: "export default async function init() {}\nexport function set_web_theme(theme) { document.body.dataset.wasmTheme = theme; }\n",
  }));
}

async function recordViolations(page: Page): Promise<string[]> {
  const consoleErrors: string[] = [];
  page.on("console", (message) => {
    if (/content[- ]security[- ]policy|refused to/i.test(message.text())) {
      consoleErrors.push(message.text());
    }
  });
  await page.addInitScript(() => {
    const store = window as unknown as { __cspViolations?: Violation[] };
    store.__cspViolations = [];
    document.addEventListener("securitypolicyviolation", (event) => {
      store.__cspViolations?.push({
        directive: event.violatedDirective,
        blocked: event.blockedURI,
        sample: event.sample,
      });
    });
  });
  return consoleErrors;
}

async function violations(page: Page): Promise<Violation[]> {
  return page.evaluate(
    () => (window as unknown as { __cspViolations?: Violation[] }).__cspViolations ?? [],
  );
}

async function prepare(page: Page, path: string): Promise<string[]> {
  await installApiMocks(
    page,
    /^\/(login|register|find-password|reset-password|verify-email)$/.test(path) ? "logged-out" : "superuser",
  );
  await setUiPreferences(page, "en-US", "light");
  await page.route("**/api/forum/notifications**", (route) => route.fulfill({ json: { data: { notifications: [], next_cursor: null } } }));
  await page.route("**/api/live-chat/messages**", (route) => route.fulfill({ json: { data: { items: [], has_more: false, next_before_message_id: null } } }));
  await page.routeWebSocket("**/ws/**", (socket) => socket.onMessage(() => {}));
  await serveSandboxedMap(page);
  if (existsSync(eu5HostPath)) await serveSandboxedEu5(page);
  await serveWithApplicationPolicy(page);
  return recordViolations(page);
}

for (const path of routes) {
  test(`application policy allows ${path}`, async ({ page }) => {
    const consoleErrors = await prepare(page, path);
    await page.goto(path, { waitUntil: "networkidle" });
    await expect(page.locator("[data-site-bar='top']")).toBeVisible();
    expect(await violations(page)).toEqual([]);
    expect(consoleErrors).toEqual([]);
  });
}

test("the harness reports violations it is meant to catch", async ({ page }) => {
  await prepare(page, "/");
  await page.goto("/", { waitUntil: "networkidle" });
  await page.evaluate(() => {
    const script = document.createElement("script");
    script.textContent = "window.__inlineRan = true;";
    document.head.append(script);
  });
  const recorded = await violations(page);
  expect(recorded.some((violation) => violation.directive.startsWith("script-src"))).toBe(true);
  expect(await page.evaluate(() => "__inlineRan" in window)).toBe(false);
});

function frameAt(page: Page, suffix: string): Frame | undefined {
  return page.frames().find((frame) => frame.url().endsWith(suffix));
}

test("the map runs in an opaque origin with in-memory storage", async ({ page }) => {
  const consoleErrors = await prepare(page, "/minecraft");
  await page.goto("/minecraft", { waitUntil: "networkidle" });
  await expect(page.frameLocator(".minecraft-map").getByRole("heading")).toHaveText("World map");
  const frame = frameAt(page, "/minecraft/map/");
  expect(frame).toBeDefined();
  expect(await frame?.evaluate(() => document.body.dataset.probe)).toBe("true null");
  // The opaque origin cannot read the parent's storage or cookies.
  expect(await frame?.evaluate(() => {
    try {
      return window.parent.document.title;
    } catch {
      return "blocked";
    }
  })).toBe("blocked");
  expect(consoleErrors).toEqual([]);
});

test("the EU5 app completes its theme handshake from an opaque origin", async ({ page }) => {
  test.skip(!existsSync(eu5HostPath), "EU5 submodule is not initialized");
  const consoleErrors = await prepare(page, "/eu5-locations-db");
  await page.goto("/eu5-locations-db", { waitUntil: "networkidle" });
  const frame = frameAt(page, "/eu5-locations-db/app/index.html");
  expect(frame).toBeDefined();
  await expect.poll(() => frame?.evaluate(() => document.documentElement.dataset.theme)).toBe("light");
  await expect.poll(() => frame?.evaluate(() => document.body.dataset.wasmTheme)).toBe("light");
  expect(await frame?.evaluate(() => self.origin)).toBe("null");
  expect(consoleErrors).toEqual([]);
});
