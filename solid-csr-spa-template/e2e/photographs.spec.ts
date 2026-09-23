import { expect, test, type Page, type Route } from "@playwright/test";
import { installApiMocks, type AuthMode } from "./fixtures";

const ADMIN_CHUNKS = /BatchUploadFields|ProcessingModal|leaflet-geosearch/;
const NOW = "2026-09-03T08:00:00.000Z";

type Gate = { readonly wait: Promise<void>; readonly open: () => void };

function gate(): Gate {
  let open = () => {};
  const wait = new Promise<void>((resolve) => (open = resolve));
  return { wait, open };
}

/** A photograph whose images are served over HTTP so tests can delay them. */
const photo = (index: number, comment = `Photo ${index}`) => ({
  photograph_comments: comment,
  photograph_context: "photography",
  photograph_created_at: NOW,
  photograph_id: `00000000-0000-4000-8000-${String(index).padStart(12, "0")}`,
  photograph_image_type: 1,
  photograph_is_on_cloud: true,
  photograph_lat: 39.7392,
  photograph_link: `/e2e-photos/${index}-full.svg`,
  photograph_lon: -104.9903,
  photograph_shot_at: NOW,
  photograph_thumbnail_link: `/e2e-photos/${index}-thumb.svg`,
  photograph_total_downvotes: 0,
  photograph_total_upvotes: 3,
  photograph_updated_at: NOW,
  photograph_view_count: 7,
  user_id: "11111111-1111-4111-8111-111111111111",
});

const photoPage = (items: ReturnType<typeof photo>[], hasNext: boolean) => ({
  success: true,
  data: {
    items,
    pagination: {
      has_next: hasNext,
      has_prev: false,
      page: 1,
      page_size: 24,
      total_items: items.length,
      total_pages: hasNext ? 2 : 1,
    },
  },
  meta: { time_to_process: "1ms" },
});

const svg = (label: string, width: number, height: number) =>
  `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}"><rect width="100%" height="100%" fill="#0f766e"/><text x="50%" y="50%" fill="#fff" font-size="${Math.round(height / 5)}" text-anchor="middle" dominant-baseline="middle">${label}</text></svg>`;

/** Serves `/e2e-photos/*`, holding any image whose name matches a gate key. */
async function serveImages(
  page: Page,
  gates: Readonly<Record<string, Gate>> = {},
  failing: readonly string[] = [],
): Promise<string[]> {
  const requested: string[] = [];
  await page.route("**/e2e-photos/**", async (route: Route) => {
    const name = new URL(route.request().url()).pathname.split("/").pop() ?? "";
    requested.push(name);
    for (const [key, held] of Object.entries(gates)) {
      if (name.startsWith(key)) await held.wait;
    }
    if (failing.some((key) => name.startsWith(key))) {
      await route.fulfill({ status: 404, body: "" });
      return;
    }
    await route.fulfill({
      contentType: "image/svg+xml",
      body: svg(name.replace(".svg", ""), 1200, 800),
    });
  });
  return requested;
}

async function requestedModules(page: Page, authMode: AuthMode): Promise<string[]> {
  const urls: string[] = [];
  page.on("request", (request) => urls.push(request.url()));
  await page.setViewportSize({ width: 1440, height: 900 });
  await installApiMocks(page, authMode);
  await page.goto("/photographs", { waitUntil: "networkidle" });
  await expect(page.locator(".photo-card").first()).toBeVisible();
  return urls;
}

test("desktop visitors do not prefetch superuser upload code", async ({ page }) => {
  const urls = await requestedModules(page, "logged-out");
  expect(urls.filter((url) => ADMIN_CHUNKS.test(url))).toEqual([]);
  expect(urls.some((url) => url.includes("PhotographMap"))).toBe(true);
});

test("desktop superusers prefetch upload and processing code", async ({ page }) => {
  const urls = await requestedModules(page, "superuser");
  await expect
    .poll(() => urls.filter((url) => ADMIN_CHUNKS.test(url)).length)
    .toBeGreaterThan(0);
});

test("appending a gallery page keeps the existing cards and images", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await installApiMocks(page, "logged-out");
  await serveImages(page);
  const secondPage = gate();
  await page.route("**/api/photographs/get**", async (route) => {
    const pageNumber = new URL(route.request().url()).searchParams.get("page");
    if (pageNumber === "1") {
      await route.fulfill({ json: photoPage([photo(1), photo(2), photo(3)], true) });
      return;
    }
    await secondPage.wait;
    await route.fulfill({ json: photoPage([photo(4), photo(5)], false) });
  });

  await page.goto("/photographs");
  await expect(page.locator(".photo-card")).toHaveCount(3);
  await page.evaluate(() => {
    const store = window as unknown as { __firstCards: Element[] };
    store.__firstCards = [...document.querySelectorAll(".photo-card, .photo-card img")];
  });
  secondPage.open();
  await expect(page.locator(".photo-card")).toHaveCount(5);
  const kept = await page.evaluate(() =>
    (window as unknown as { __firstCards: Element[] }).__firstCards.map(
      (element) => element.isConnected,
    ),
  );
  expect(kept).toHaveLength(6);
  expect(kept.every(Boolean)).toBe(true);
});

test("thumbnails reserve a 4:3 box before their image arrives", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await installApiMocks(page, "logged-out");
  const held = gate();
  await serveImages(page, { "1-thumb": held });
  await page.route("**/api/photographs/get**", (route) =>
    route.fulfill({ json: photoPage([photo(1)], false) }),
  );
  await page.goto("/photographs");
  const thumbnail = page.locator(".photo-card img").first();
  await expect(thumbnail).toBeAttached();
  const box = await thumbnail.boundingBox();
  expect(box).not.toBeNull();
  expect(Math.abs((box?.height ?? 0) - (box?.width ?? 0) * 0.75)).toBeLessThan(2);
  held.open();
  await expect
    .poll(async () => {
      const loaded = await thumbnail.boundingBox();
      return Math.round(((loaded?.height ?? 0) / (loaded?.width ?? 1)) * 100);
    })
    .toBe(67);
});
