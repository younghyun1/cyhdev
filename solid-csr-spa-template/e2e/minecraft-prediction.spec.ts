import { expect, test } from "@playwright/test";
import { installApiMocks, setUiPreferences } from "./fixtures";
import { installMinecraftMapMocks } from "./minecraft-fixtures";

for (const width of [1440, 390]) {
  test(`continuous seed tiles pan and zoom without clearing loaded neighbors at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 1000 });
    await installApiMocks(page, "logged-out"); await setUiPreferences(page, "en-US", "light");
    const fixture = await installMinecraftMapMocks(page);
    const errors: string[] = [];
    page.on("pageerror", error => errors.push(error.message));
    page.on("console", message => { if (message.text().includes("STRICT_READ_UNTRACKED")) errors.push(message.text()); });
    await page.goto("/minecraft");
    const ready = page.locator('.minecraft-seed-tile[data-ready="true"]');
    await expect(ready.first()).toBeVisible();
    await expect.poll(() => fixture.predictions.length).toBeGreaterThan(4);
    expect(fixture.predictions.every(query => query.world === "minecraft:overworld" && query.y === 64)).toBe(true);
    expect(fixture.predictions.some(query => query.tile_x < 0 || query.tile_z < 0)).toBe(true);
    await expect(page.locator(".minecraft-prediction-cell")).toHaveCount(0);
    await expect(page.locator(".leaflet-minecraft-seeds-pane")).toHaveCSS("z-index", "190");
    await expect(page.locator(".leaflet-tile-pane")).toHaveCSS("z-index", "200");
    const original = await ready.first().elementHandle(), box = await page.locator(".minecraft-atlas-canvas").boundingBox();
    if (!box) throw new Error("Missing terrain canvas");
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down(); await page.mouse.move(box.x + box.width / 2 - 140, box.y + box.height / 2, { steps: 8 }); await page.mouse.up();
    expect(await original?.evaluate(node => node.isConnected)).toBe(true);
    await expect(ready.first()).toBeVisible();
    await page.getByRole("button", { name: "Zoom out" }).click();
    await expect(page.locator(".minecraft-atlas-canvas")).toHaveAttribute("data-zoom", "3");
    await page.getByRole("button", { name: "Zoom out" }).click();
    await expect.poll(() => fixture.predictions.some(query => query.level > 0)).toBe(true);
    await expect(ready.first()).toBeVisible();
    await page.getByRole("button", { name: "Layers", exact: true }).click();
    await expect(page.getByLabel("Predicted biomes", { exact: true })).toBeChecked();
    await page.getByLabel("Prediction Y", { exact: true }).fill("-16");
    await expect(page.locator(".minecraft-prediction-receipt")).toContainText("Predicted · Y -16 · Large biomes");
    await expect.poll(() => fixture.predictions.some(query => query.y === -16)).toBe(true);
    await page.getByRole("button", { name: "Close map menu" }).click();
    await page.screenshot({ path: `../target/minecraft-tests/continuous-seed-${width}.png` });
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(width);
    await page.getByLabel("Dimension", { exact: true }).selectOption("minecraft_the_nether");
    await expect(page.locator(".minecraft-seed-tile")).toHaveCount(0);
    expect(errors).toEqual([]);
  });
}

test("permission refresh retains valid imagery and removes it at expiry when renewal fails", async ({ page }) => {
  await page.clock.install();
  await installApiMocks(page, "logged-out"); await setUiPreferences(page, "en-US", "light");
  const fixture = await installMinecraftMapMocks(page);
  await page.goto("/minecraft");
  const ready = page.locator('.minecraft-seed-tile[data-ready="true"]');
  await expect(ready.first()).toBeVisible();
  await page.clock.pauseAt(new Date(Date.now() + 1000));
  await page.route("**/api/minecraft/map/seed-tile", route => route.fulfill({ status: 503, json: { error_code: 89, message: "Unavailable" } }));
  const count = fixture.predictions.length;
  await page.clock.runFor(11000);
  await expect(ready.first()).toBeVisible();
  await page.clock.runFor(5000);
  await expect(ready).toHaveCount(0);
  await page.getByRole("button", { name: "Layers", exact: true }).click();
  await expect(page.getByText("Seed map is temporarily unavailable.")).toBeVisible();
  await page.getByLabel("Predicted biomes", { exact: true }).uncheck();
  await page.clock.runFor(20000);
  expect(fixture.predictions.length).toBe(count);
  await expect(page.locator(".minecraft-seed-tile")).toHaveCount(0);
});

test("actual terrain remains above seed colors and supplies observed hover data", async ({ page }) => {
  await page.clock.install();
  await installApiMocks(page, "logged-out"); await setUiPreferences(page, "en-US", "light");
  const fixture = await installMinecraftMapMocks(page);
  await page.goto("/minecraft");
  await expect(page.locator('.minecraft-seed-tile[data-ready="true"]').first()).toBeVisible();
  const box = await page.locator(".minecraft-atlas-canvas").boundingBox(); if (!box) throw new Error("Missing terrain canvas");
  await page.mouse.move(box.x + Math.floor(box.width / 2) + 13, box.y + Math.floor(box.height / 2) + 5);
  await page.clock.runFor(1600);
  await expect(page.getByRole("region", { name: "Terrain inspection" })).toContainText("Observed biome:");
  expect(fixture.queries.some(query => query.kind === "area" && query.width === 1)).toBe(true);
});

test("frontier has one thin line across actual tiles and no internal tile seams", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await installApiMocks(page, "logged-out"); await setUiPreferences(page, "en-US", "light");
  await installMinecraftMapMocks(page);
  await page.route("**/minecraft/map/tiles/**/*.png*", route => {
    const parts = /\/(\d+)\/(-?\d+)_(-?\d+)\.png/.exec(route.request().url());
    if (!parts) return route.fallback();
    const step = 2 ** (3 - Number(parts[1])), min = Number(parts[2]) * 512 * step;
    const width = Math.max(0, Math.min(512, (192 - min) / step));
    return route.fulfill({ contentType: "image/svg+xml", body: `<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512"><defs><pattern id="grass" width="16" height="16" patternUnits="userSpaceOnUse"><rect width="16" height="16" fill="#799452"/><rect width="6" height="5" fill="#859d60"/></pattern></defs><rect width="${width}" height="512" fill="url(#grass)"/></svg>` });
  });
  await page.goto("/minecraft");
  await expect(page.locator('.minecraft-seed-tile[data-ready="true"]').first()).toBeVisible();
  const frontier = page.locator(".leaflet-minecraft-frontier-pane canvas");
  await expect(frontier).toHaveCount(1);
  await expect.poll(() => frontier.evaluate((node: HTMLCanvasElement) => {
    const data = node.getContext("2d")?.getImageData(0, 0, node.width, node.height).data;
    if (!data) return 0;
    const columns = new Set<number>();
    for (let i = 3; i < data.length; i += 4) if ((data[i] ?? 0) > 0) columns.add(Math.floor(i / 4) % node.width);
    return columns.size;
  })).toBeLessThanOrEqual(2);
  await expect.poll(() => frontier.evaluate((node: HTMLCanvasElement) => node.getContext("2d")?.getImageData(0, 0, node.width, node.height).data.some((value, i) => i % 4 === 3 && value > 0))).toBe(true);
  await expect.poll(() => page.locator(".minecraft-seed-tile").evaluateAll(nodes => nodes.every(node => node.getAttribute("data-ready") === "true" && getComputedStyle(node).opacity === "1"))).toBe(true);
  await page.mouse.move(1, 1);
  await page.screenshot({ path: "../target/minecraft-tests/continuous-seed-frontier-1440.png" });
});

test("failed tiles finish their lifecycle so subsequent zooms can load", async ({ page }) => {
  await page.clock.install();
  await installApiMocks(page, "logged-out"); await setUiPreferences(page, "en-US", "light");
  await installMinecraftMapMocks(page);
  let failing = true;
  await page.route("**/api/minecraft/map/seed-tile", route => failing ? route.fulfill({ status: 503, json: { error_code: 89 } }) : route.fallback());
  await page.goto("/minecraft");
  await expect(page.locator(".minecraft-seed-tile").first()).toBeAttached();
  await page.clock.runFor(13000);
  await expect(page.locator(".minecraft-seed-tile.leaflet-tile-loaded").first()).toBeAttached();
  failing = false;
  await page.getByRole("button", { name: "Zoom out" }).click(); await page.clock.runFor(500);
  await page.getByRole("button", { name: "Zoom out" }).click(); await page.clock.runFor(500);
  await expect(page.locator('.minecraft-seed-tile[data-ready="true"]').first()).toBeVisible();
});
