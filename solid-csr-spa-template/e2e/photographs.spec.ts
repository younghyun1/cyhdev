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
  failing: ReadonlySet<string> = new Set(),
): Promise<string[]> {
  const requested: string[] = [];
  await page.route("**/e2e-photos/**", async (route: Route) => {
    const name = new URL(route.request().url()).pathname.split("/").pop() ?? "";
    requested.push(name);
    for (const [key, held] of Object.entries(gates)) {
      if (name.startsWith(key)) await held.wait;
    }
    if ([...failing].some((key) => name.startsWith(key))) {
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

async function openViewer(page: Page, items: ReturnType<typeof photo>[]) {
  await page.route("**/api/photographs/get**", (route) =>
    route.fulfill({ json: photoPage(items, false) }),
  );
  await page.goto("/photographs");
  await page.locator(".photo-card").first().click();
  await expect(page.locator(".details-modal")).toBeVisible();
}

async function servePhotoDetails(page: Page, items: ReturnType<typeof photo>[]) {
  await page.route("**/api/photographs/*", async (route) => {
    const id = new URL(route.request().url()).pathname.split("/").pop();
    const item = items.find((entry) => entry.photograph_id === id);
    if (!item) return route.fallback();
    await route.fulfill({
      json: {
        success: true,
        data: { photograph: item, comments: [], vote_state: 2 },
        meta: { time_to_process: "1ms" },
      },
    });
  });
}

test("comment caret arrows keep the photograph and draft selected", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await installApiMocks(page, "authenticated");
  await serveImages(page);
  const items = [photo(1), photo(2)];
  await servePhotoDetails(page, items);
  await openViewer(page, items);
  const composer = page.locator(".details-modal textarea");
  await composer.fill("Comment draft");
  await composer.press("ArrowLeft");
  await composer.press("ArrowRight");
  await expect(page).toHaveURL(new RegExp(`${photo(1).photograph_id}$`));
  await expect(composer).toHaveValue("Comment draft");
  await expect(composer).toBeFocused();

  // The same shortcut still navigates when focus is outside an editable field.
  await page.locator("[data-photo-close]").focus();
  await page.keyboard.press("ArrowRight");
  await expect(page).toHaveURL(new RegExp(`${photo(2).photograph_id}$`));
  await expect(composer).toHaveValue("");
});

test("photo votes and pending failures belong to the photograph that started them", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await installApiMocks(page, "authenticated");
  await serveImages(page);
  const items = [photo(1), photo(2)];
  await servePhotoDetails(page, items);
  const firstVote = gate();
  const mutations: { method: string; path: string }[] = [];
  await page.route("**/api/photographs/*/vote", async (route) => {
    const path = new URL(route.request().url()).pathname;
    mutations.push({ method: route.request().method(), path });
    if (path.includes(photo(1).photograph_id)) {
      await firstVote.wait;
      await route.fulfill({ status: 500, json: { success: false } });
    } else {
      await route.fulfill({ json: { success: true, data: null } });
    }
  });
  await openViewer(page, items);
  const upvote = page.getByRole("button", { name: "Upvote", exact: true });
  await upvote.click();
  await expect(upvote).toBeDisabled();
  await expect.poll(() => mutations.length).toBe(1);
  await page.keyboard.press("ArrowRight");
  await expect(page).toHaveURL(new RegExp(`${photo(2).photograph_id}$`));
  await expect(upvote).toBeEnabled();
  await expect(upvote).not.toHaveClass(/font-bold/);
  await upvote.click();
  await expect(upvote).toBeEnabled();
  await expect(upvote).toHaveClass(/font-bold/);
  expect(mutations).toEqual(items.map((item) => ({
    method: "POST",
    path: `/api/photographs/${item.photograph_id}/vote`,
  })));

  const failed = page.waitForResponse((response) =>
    response.url().includes(`${photo(1).photograph_id}/vote`) && response.status() === 500,
  );
  firstVote.open();
  await failed;
  await page.waitForLoadState("networkidle");
  await expect(upvote).toHaveClass(/font-bold/);
});

test("a pending comment cannot appear on the next photograph", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await installApiMocks(page, "authenticated");
  await serveImages(page);
  const items = [photo(1), photo(2)];
  await servePhotoDetails(page, items);
  const pending = gate();
  await page.route(`**/api/photographs/${photo(1).photograph_id}/comment`, async (route) => {
    await pending.wait;
    await route.fulfill({ json: {
      success: true,
      data: {
        photograph_id: photo(1).photograph_id,
        photograph_comment_id: "late-comment",
        photograph_comment_content: "Only on the first photo",
        photograph_comment_created_at: NOW,
        photograph_comment_total_upvotes: 0,
        photograph_comment_total_downvotes: 0,
        user_id: photo(1).user_id,
        user_name: "Comment author",
        vote_state: 2,
      },
    } });
  });
  await openViewer(page, items);
  const composer = page.locator(".details-modal textarea");
  await composer.fill("Only on the first photo");
  const requested = page.waitForRequest((request) => request.url().endsWith("/comment"));
  await page.locator(".details-modal form button[type=submit]").click();
  await requested;
  await page.locator("[data-photo-close]").focus();
  await page.keyboard.press("ArrowRight");
  await expect(page).toHaveURL(new RegExp(`${photo(2).photograph_id}$`));
  await expect(composer).toHaveValue("");
  await composer.fill("Second photo draft");
  const completed = page.waitForResponse((response) => response.url().endsWith("/comment"));
  pending.open();
  await completed;
  await page.waitForLoadState("networkidle");
  await expect(page.locator(".details-modal")).not.toContainText("Only on the first photo");
  await expect(composer).toHaveValue("Second photo draft");
});

for (const [initialWidth, resizedWidth, pageSize] of [[390, 1440, 12], [1440, 390, 24]] as const) {
  test(`gallery pagination stays contiguous when resizing from ${initialWidth}px to ${resizedWidth}px`, async ({ page }) => {
    await page.setViewportSize({ width: initialWidth, height: 400 });
    await installApiMocks(page, "logged-out");
    await serveImages(page);
    const items = Array.from({ length: 48 }, (_, index) => photo(index + 1));
    const requests: { page: number; pageSize: number }[] = [];
    await page.route("**/api/photographs/get**", async (route) => {
      const query = new URL(route.request().url()).searchParams;
      const requestedPage = Number(query.get("page"));
      const requestedSize = Number(query.get("page_size"));
      requests.push({ page: requestedPage, pageSize: requestedSize });
      const offset = (requestedPage - 1) * requestedSize;
      await route.fulfill({ json: photoPage(items.slice(offset, offset + requestedSize), requestedPage < 2) });
    });
    await page.goto("/photographs");
    await expect(page.locator(".photo-card")).toHaveCount(pageSize);
    await page.setViewportSize({ width: resizedWidth, height: 400 });
    await page.locator("#scroll-sentinel").scrollIntoViewIfNeeded();
    await expect(page.locator(".photo-card")).toHaveCount(pageSize * 2);
    expect(requests).toEqual([{ page: 1, pageSize }, { page: 2, pageSize }]);
    expect(await page.locator(".photo-card img").evaluateAll((images) =>
      images.map((image) => image.getAttribute("alt")).sort(),
    )).toEqual(items.slice(0, pageSize * 2).map((item) => item.photograph_comments).sort());
  });
}

const stageImage = (page: Page, index: number) =>
  page.locator(`.details-image-container img[src$="/${index}-full.svg"]`);

test("viewer shows a black stage and a delayed indicator until the next photo decodes", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await installApiMocks(page, "logged-out");
  const second = gate();
  await serveImages(page, { "2-full": second });
  await openViewer(page, [photo(1, "First photo"), photo(2, "Second photo")]);
  await expect(stageImage(page, 1)).toBeVisible();
  await expect(page.locator("[data-photo-loading]")).toHaveCount(0);

  await page.keyboard.press("ArrowRight");
  await expect(page.locator(".details-info")).toContainText("Second photo");
  // Read description and stage together so no frame can show both.
  const snapshot = await page.evaluate(() => ({
    description: document.querySelector(".details-info p")?.textContent ?? "",
    images: [...document.querySelectorAll(".details-image-container img")].map(
      (image) => image.getAttribute("src"),
    ),
  }));
  expect(snapshot.description).toContain("Second photo");
  expect(snapshot.images).toEqual([]);
  await expect(page.locator("[data-photo-loading]")).toBeVisible();
  await expect(page.getByRole("status").filter({ hasText: "Loading photograph" })).toHaveCount(1);

  second.open();
  await expect(stageImage(page, 2)).toBeVisible();
  await expect(page.locator("[data-photo-loading]")).toHaveCount(0);
  await expect(stageImage(page, 1)).toHaveCount(0);

  // Going back to an already decoded photo is immediate: no indicator at all.
  await page.evaluate(() => {
    const store = window as unknown as { __sawIndicator: boolean };
    store.__sawIndicator = false;
    new MutationObserver(() => {
      if (document.querySelector("[data-photo-loading]")) store.__sawIndicator = true;
    }).observe(document.body, { childList: true, subtree: true });
  });
  await page.keyboard.press("ArrowLeft");
  await expect(stageImage(page, 1)).toBeVisible();
  await page.waitForTimeout(300);
  expect(
    await page.evaluate(() => (window as unknown as { __sawIndicator: boolean }).__sawIndicator),
  ).toBe(false);

  await page.keyboard.press("Escape");
  await expect(page.locator(".details-modal")).toBeHidden();
  await expect(page.locator(".photo-card").first()).toBeFocused();
});

test("viewer preloads the adjacent photographs after the current one", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await installApiMocks(page, "logged-out");
  const requested = await serveImages(page);
  await page.route("**/api/photographs/get**", (route) =>
    route.fulfill({ json: photoPage([photo(1), photo(2), photo(3), photo(4)], false) }),
  );
  await page.goto("/photographs");
  await page.locator(".photo-card").nth(1).click();
  await expect(stageImage(page, 2)).toBeVisible();
  await expect.poll(() => requested.filter((name) => name.endsWith("-full.svg")).sort())
    .toEqual(["1-full.svg", "2-full.svg", "3-full.svg"]);
  await page.keyboard.press("ArrowRight");
  await expect(stageImage(page, 3)).toBeVisible();
  await expect(page.locator("[data-photo-loading]")).toHaveCount(0);
  await expect.poll(() => requested.includes("4-full.svg")).toBe(true);
});

test("viewer reports a failed photograph and retries it", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installApiMocks(page, "logged-out");
  const failing = new Set(["1-full"]);
  const secondRetry = gate();
  await serveImages(page, {}, failing);
  await openViewer(page, [photo(1, "Broken photo"), photo(2, "Working photo")]);
  const alert = page.getByRole("alert");
  await expect(alert).toContainText("This photograph could not be loaded.");
  await expect(page.locator(".details-info")).toContainText("Broken photo");
  const retry = alert.getByRole("button", { name: "Try again" });
  expect((await retry.boundingBox())?.height ?? 0).toBeGreaterThanOrEqual(44);

  // Coming back to the failed photo retries it and shows that as loading.
  await page.keyboard.press("ArrowRight");
  await expect(stageImage(page, 2)).toBeVisible();
  await page.route("**/e2e-photos/1-full.svg", async (route) => {
    await secondRetry.wait;
    await route.fallback();
  });
  await page.keyboard.press("ArrowLeft");
  await expect(page.locator(".details-info")).toContainText("Broken photo");
  await expect(page.locator("[data-photo-loading]")).toBeVisible();
  await expect(alert).toHaveCount(0);
  secondRetry.open();
  await expect(alert).toContainText("This photograph could not be loaded.");

  failing.clear();
  await retry.click();
  await expect(stageImage(page, 1)).toBeVisible();
  await expect(alert).toHaveCount(0);
});

test("viewer loading cues are still under reduced motion", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.setViewportSize({ width: 1440, height: 900 });
  await installApiMocks(page, "logged-out");
  const held = gate();
  await serveImages(page, { "1-full": held });
  await openViewer(page, [photo(1)]);
  const indicator = page.locator("[data-photo-loading]");
  await expect(indicator).toBeVisible();
  expect(
    await indicator.evaluate((element) => getComputedStyle(element, "::before").animationName),
  ).toBe("none");
  held.open();
  const image = stageImage(page, 1);
  await expect(image).toBeVisible();
  expect(await image.evaluate((element) => getComputedStyle(element).animationName)).toBe("none");
});
