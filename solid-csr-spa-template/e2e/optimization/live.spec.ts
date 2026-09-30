import { writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, test, type Browser } from "@playwright/test";
import { benchmark } from "./benchmark";
import { currentRoutes, pagePath } from "./inventory";
import { start, stop } from "./lifecycle";
import { digest, interpolate, localUrl, readJson, required, validate, type Actor, type Campaign } from "./model";
import { execute, operationMatches } from "./steps";
import { actorContext } from "./sessions";

const policy = readJson<{ pages: string[]; scenarios: string[] }>(resolve(import.meta.dirname, "../../../tools/optimization/coverage.json"));
const campaign = readJson<Campaign>(required("CYHDEV_OPT_CAMPAIGN"));
validate(campaign);
const inventory = currentRoutes();
test("comprehensive live optimization campaign", async ({ browser, request }) => {
  expect(inventory.map((route) => route.pattern).sort()).toEqual([...policy.pages].sort());
  if (digest(required("CYHDEV_BINARY")) !== required("CYHDEV_BINARY_SHA256")
    || digest(required("CYHDEV_OPT_CAMPAIGN")) !== required("CYHDEV_CAMPAIGN_SHA256")) throw new Error("Campaign identity changed");
  const child = await start(campaign, request);
  let complete = false;
  try {
    if (process.env.CYHDEV_OPT_MODE === "benchmark") { await benchmark(browser, campaign); }
    else { await train(browser); }
    complete = true;
  } finally {
    await stop(child);
    if (!complete) writeFileSync(required("CYHDEV_COVERAGE_REPORT"), JSON.stringify({ failures: 1 }), { mode: 0o600 });
  }
});

async function train(browser: Browser): Promise<void> {
  const surface = readJson<{ operations: string[] }>(resolve(required("CYHDEV_OPT_RUN"), "surface.json"));
  const declared = new Set(campaign.workflows.flatMap((flow) => flow.steps.flatMap((step) => step.kind === "request" && step.status < 400 ? [`${step.method} ${step.route}`] : [])));
  const missing = surface.operations.filter((operation) => !declared.has(operation));
  const missingFlows = policy.scenarios.filter((name) => !campaign.workflows.some((flow) => flow.name === name));
  if (missing.length || missingFlows.length) throw new Error(`Incomplete campaign configuration; API operations: ${missing.join(", ")}; scenarios: ${missingFlows.join(", ")}`);
  const operations = new Set<string>();
  const pages = new Set<string>();
  const scenarios = new Set<string>();
  const states = new Map<Actor, Awaited<ReturnType<Awaited<ReturnType<typeof actorContext>>["storageState"]>>>();
  for (const actor of ["anonymous", "member", "admin"] as const) {
    const context = await actorContext(browser, campaign, actor);
    try { states.set(actor, await context.storageState()); } finally { await context.close(); }
  }
  // Desktop/mobile, light/dark, both maintained UI locales, real session authority.
  for (const route of inventory) {
    const fixture = campaign.pages[route.pattern];
    if (!fixture?.selector) throw new Error(`Missing positive page assertion for ${route.pattern}`);
    for (const width of [1440, 390]) for (const theme of ["light", "dark"]) for (const locale of ["en-US", "ko-KR"]) {
      const actor = fixture.actor ?? route.actor;
      if ((route.actor === "admin" && actor !== "admin") || (route.actor === "member" && actor === "anonymous"))
        throw new Error(`Positive page coverage requires an authorized actor for ${route.pattern}`);
      const context = await browser.newContext({ ignoreHTTPSErrors: true, storageState: states.get(actor), viewport: { width, height: width === 390 ? 844 : 900 } });
      try {
        await context.addInitScript(({ theme, locale }) => {
          try { localStorage.setItem("theme", theme); localStorage.setItem("ui_locale", locale); } catch { /* Opaque embedded origins do not expose storage. */ }
        }, { theme, locale });
        const page = await context.newPage();
        const errors: string[] = [];
        page.on("pageerror", () => errors.push("pageerror"));
        const path = interpolate(fixture.path ?? pagePath(route.pattern, campaign.parameters), campaign.parameters);
        const matches = (pattern: string) => operationMatches(pattern.replace(/:([A-Za-z_][A-Za-z0-9_]*)/g, "{$1}"), path);
        if (route.pattern === "*404" ? inventory.some(({ pattern }) => pattern !== "*404" && matches(pattern)) : !matches(route.pattern))
          throw new Error(`Page fixture path does not match ${route.pattern}`);
        const response = await page.goto(localUrl(campaign.base_url, path), { waitUntil: "domcontentloaded" });
        if (!response?.ok()) throw new Error(`Page response failed for ${route.pattern}`);
        // Redirecting to sign-in is not a successful visit to a protected page.
        expect(new URL(page.url()).pathname).toBe(new URL(path, campaign.base_url).pathname);
        await expect(page.locator(fixture.selector)).toBeVisible();
        if (fixture.text) await expect(page.locator(fixture.selector)).toContainText(fixture.text);
        expect(errors).toEqual([]);
      } finally { await context.close(); }
    }
    pages.add(route.pattern);
  }
  const variables = { ...campaign.parameters };
  for (const flow of campaign.workflows) {
    const context = await actorContext(browser, campaign, flow.actor);
    try {
      const page = await context.newPage();
      for (let repetition = 0; repetition < (flow.repetitions ?? 1); repetition += 1)
        for (const step of flow.steps) await execute(context, page, campaign, step, variables, operations);
      scenarios.add(flow.name);
    } finally { await context.close(); }
  }
  // Receipts contain identities and coverage only, never response bodies or sessions.
  writeFileSync(required("CYHDEV_COVERAGE_REPORT"), JSON.stringify({ schema_version: 1, stage: required("CYHDEV_OPT_STAGE"),
    binary_sha256: required("CYHDEV_BINARY_SHA256"), campaign_sha256: required("CYHDEV_CAMPAIGN_SHA256"),
    operations: [...operations].sort(), pages: [...pages].sort(), scenarios: [...scenarios].sort(), failures: 0 }, null, 2), { mode: 0o600 });
}
