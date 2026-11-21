import { expect } from "@playwright/test";
import { pointer } from "../model";
import { another, call, type Scene } from "./support";

const BASE = "/api/admin/kcna-archive";
const CSV_HEADER = "id,date,language,sources,category,title\r\n";
const KOREAN_LANTERN = encodeURIComponent("등불");

/** Counts of the synthetic corpus come from the fixture receipt, not from this file. */
function expected(scene: Scene, name: string): number {
  const count = Number(scene.variables[name]);
  if (!Number.isInteger(count) || count < 0) throw new Error(`Missing synthetic archive fixture count ${name}`);
  return count;
}
function list(json: unknown, path: string): unknown[] {
  const found = pointer(json, path);
  if (!Array.isArray(found)) throw new Error(`Expected a list at ${path}`);
  return found;
}
function total(values: unknown[]): number {
  return values.reduce<number>((sum, item) => sum + (typeof item === "number" ? item : Number.NaN), 0);
}
const denied = { json_pointer: undefined, minimum_bytes: 1 };

export async function kcna(name: string, scene: Scene): Promise<void> {
  if (name !== "kcna.search-read-trends-reload") throw new Error(`Unknown KCNA archive scenario ${name}`);
  const indexed = expected(scene, "kcna_indexed");
  const lantern = expected(scene, "kcna_lantern_total");
  const years = expected(scene, "kcna_years");

  const status = await call(scene, "GET", `${BASE}/status`, undefined, { json_pointer: "/data/state", equals: "ready" });
  expect(pointer(status, "/data/snapshot/indexed")).toBe(indexed);
  expect(pointer(status, "/data/reloading")).toBe(false);
  expect(pointer(status, "/data/snapshot/translationGroups")).toBeGreaterThan(0);

  // The oldest match is the English edition that has a translation and a copy.
  const found = await call(scene, "GET", `${BASE}/search`, undefined,
    { path: `${BASE}/search?q=lantern&sort=oldest&limit=20`, capture: { kcna_document_id: "/data/items/0/id" } });
  expect(pointer(found, "/data/total")).toBe(lantern);
  expect(list(found, "/data/items")).toHaveLength(lantern);
  expect((pointer(found, "/data/items/0/title") as { hit: boolean }[]).some((segment) => segment.hit)).toBe(true);
  expect(scene.variables.kcna_document_id).toBe(scene.variables.documentId);
  const korean = await call(scene, "GET", `${BASE}/search`, undefined,
    { path: `${BASE}/search?q=${KOREAN_LANTERN}&lang=kor` });
  expect(pointer(korean, "/data/total")).toBe(expected(scene, "kcna_korean_lantern_total"));
  const filtered = await call(scene, "GET", `${BASE}/search`, undefined,
    { path: `${BASE}/search?q=lantern&lang=eng&source=kcna_kp&category=notice&from=2019-01-01&to=2026-12-31&sort=relevance` });
  expect(pointer(filtered, "/data/total")).toBeGreaterThan(0);
  expect(pointer(filtered, "/data/total")).toBeLessThan(lantern);

  const histogram = await call(scene, "GET", `${BASE}/histogram`, undefined, { path: `${BASE}/histogram?bucket=year` });
  const buckets = list(histogram, "/data/buckets") as { start: string; count: number }[];
  expect(buckets).toHaveLength(years);
  expect(total(buckets.map((bucket) => bucket.count))).toBe(indexed);
  expect(buckets.map((bucket) => bucket.start)).toEqual(buckets.map((bucket) => bucket.start).sort());

  const trends = await call(scene, "GET", `${BASE}/trends`, undefined,
    { path: `${BASE}/trends?term=lantern&term=harbor&bucket=year` });
  const series = list(trends, "/data/series") as { term: string; counts: number[] }[];
  expect(series.map((entry) => entry.term)).toEqual(["lantern", "harbor"]);
  expect(list(trends, "/data/starts")).toHaveLength(years);
  expect(total(list(trends, "/data/totals"))).toBe(indexed);
  expect(series.every((entry) => entry.counts.length === years)).toBe(true);
  expect(total(series[0]?.counts ?? [])).toBe(lantern);
  expect(total(series[1]?.counts ?? [])).toBeGreaterThan(0);

  const reader = await call(scene, "GET", `${BASE}/documents/{document_id}`, undefined,
    { path: `${BASE}/documents/${scene.variables.kcna_document_id}?q=lantern`, json_pointer: "/data/id", equals: "${kcna_document_id}" });
  expect(pointer(reader, "/data/language")).toBe("eng");
  expect(list(reader, "/data/translations")).toHaveLength(1);
  expect(pointer(reader, "/data/translations/0/language")).toBe("kor");
  expect(pointer(reader, "/data/translations/0/id")).toBe(scene.variables.kcna_translation_id);
  expect(list(reader, "/data/copies")).toHaveLength(1);
  expect(pointer(reader, "/data/copies/0/source")).toBe("kcna_co_jp");
  const edition = await call(scene, "GET", `${BASE}/documents/{document_id}`, undefined,
    { path: `${BASE}/documents/${scene.variables.kcna_translation_id}`, json_pointer: "/data/id", equals: "${kcna_translation_id}" });
  expect(pointer(edition, "/data/translations/0/id")).toBe(scene.variables.kcna_document_id);

  // The download is not JSON: assert type and size through the request step, then read the header line.
  const exportPath = `${BASE}/export?q=lantern&sort=oldest`;
  await call(scene, "GET", `${BASE}/export`, undefined,
    { path: `${exportPath}&format=csv`, json_pointer: undefined, minimum_bytes: 200, content_type: "text/csv" });
  // JSON Lines is one object per line, which the request step would try to parse as a single document.
  const lines = await scene.context.request.get(`${scene.campaign.base_url}${exportPath}&format=jsonl`,
    { headers: { Origin: new URL(scene.campaign.base_url).origin } });
  try {
    expect(lines.status()).toBe(200);
    expect(lines.headers()["content-type"]).toContain("application/x-ndjson");
    const rows = (await lines.text()).trimEnd().split("\n").map((line): unknown => JSON.parse(line));
    expect(rows).toHaveLength(lantern);
    expect(pointer(rows[0], "/id")).toBe(scene.variables.kcna_document_id);
  } finally { await lines.dispose(); }
  const download = await scene.context.request.get(`${scene.campaign.base_url}${exportPath}`,
    { headers: { Origin: new URL(scene.campaign.base_url).origin } });
  try {
    expect(download.status()).toBe(200);
    expect(download.headers()["content-disposition"]).toContain("kcna-archive-export.csv");
    const text = await download.text();
    expect(text.startsWith(CSV_HEADER)).toBe(true);
    expect(text.trimEnd().split("\r\n")).toHaveLength(lantern + 1);
  } finally { await download.dispose(); }

  // Negative cases train the error and authorization paths; they never count as coverage.
  await call(scene, "GET", `${BASE}/documents/{document_id}`, undefined,
    { path: `${BASE}/documents/00000000-0000-0000-0000-0000000003e7`, status: 404, ...denied });
  await call(scene, "GET", `${BASE}/export`, undefined, { path: `${BASE}/export?format=xml`, status: 400, ...denied });
  await another(scene, "member", async (member) => {
    await call(member, "GET", `${BASE}/search`, undefined, { path: `${BASE}/search?q=lantern`, status: 403, ...denied });
    await call(member, "POST", `${BASE}/reload`, undefined, { status: 403, ...denied });
  });
  await another(scene, "anonymous", async (visitor) => {
    await call(visitor, "GET", `${BASE}/status`, undefined, { status: 401, ...denied });
  });

  await browse(scene);

  const reloaded = await call(scene, "POST", `${BASE}/reload`, undefined, { json_pointer: "/data/state", equals: "ready" });
  expect(pointer(reloaded, "/data/snapshot/indexed")).toBe(indexed);
  expect(pointer(reloaded, "/data/lastReloadFailed")).toBe(false);
  const after = await call(scene, "GET", `${BASE}/status`, undefined, { json_pointer: "/data/state", equals: "ready" });
  expect(pointer(after, "/data/snapshot/indexed")).toBe(indexed);
  expect(pointer(after, "/data/reloading")).toBe(false);
}

/** Search through the page, open the only match, and toggle its translation beside it. */
async function browse(scene: Scene): Promise<void> {
  const page = scene.page;
  // Wide enough that results open beside the list rather than on their own page.
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(`${scene.campaign.base_url}/projects/kcna-archive`);
  await expect(page.locator(".kcna-row").first()).toBeVisible();
  const field = page.locator("#kcna-search-input");
  await field.fill("Aldermoor");
  await field.press("Enter");
  await expect(page).toHaveURL(/q=Aldermoor/);
  const rows = page.locator(".kcna-row");
  await expect(rows).toHaveCount(1);
  await expect(page.locator(".kcna-row-title mark").first()).toBeVisible();
  await rows.first().click();
  await expect(page).toHaveURL(/doc=/);
  await expect(page.locator(".kcna-document-title")).toContainText("Aldermoor");
  const toggle = page.locator(".kcna-translation .kcna-toggle").first();
  await toggle.click();
  await expect(page).toHaveURL(/with=kor/);
  await expect(page.locator(".kcna-parallel-row").first()).toBeVisible();
  await toggle.click();
  await expect(page.locator(".kcna-parallel-row")).toHaveCount(0);

  await page.goto(`${scene.campaign.base_url}/projects/kcna-archive/status`);
  await expect(page.locator(".kcna-stats")).toBeVisible();
  await page.locator(".kcna-status-head .kcna-button").click();
  await expect(page.locator(".kcna-notice--ok")).toBeVisible();
}
