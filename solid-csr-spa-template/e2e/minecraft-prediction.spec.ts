import { expect, test } from "@playwright/test";
import { installApiMocks, setUiPreferences } from "./fixtures";
import { installMinecraftMapMocks } from "./minecraft-fixtures";

for (const width of [1440, 390]) {
  test(`bounded predictions show provenance and expire at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 1000 });
    await page.clock.install();
    await installApiMocks(page, "logged-out"); await setUiPreferences(page, "en-US", "light");
    const fixture = await installMinecraftMapMocks(page);
    const errors: string[] = [];
    page.on("pageerror", error => errors.push(error.message));
    page.on("console", message => { if (message.text().includes("STRICT_READ_UNTRACKED")) errors.push(message.text()); });
    await page.goto("/minecraft");
    await expect(page.getByRole("button", { name: "Preview selected area" })).toBeEnabled();
    expect(fixture.predictions).toEqual([]);
    await page.getByLabel("Prediction Y", { exact: true }).fill("-16");
    await page.getByRole("button", { name: "Preview selected area" }).click();
    await expect(page.getByText("Predicted · Y -16 · Large biomes", { exact: true })).toBeVisible();
    await expect(page.locator(".minecraft-prediction-cell")).toHaveCount(976);
    await expect(page.locator(".minecraft-prediction-boundary")).toHaveCount(1);
    await expect(page.locator(".minecraft-prediction-boundary")).toHaveCSS("stroke", "rgb(17, 21, 15)");
    await expect(page.locator(".minecraft-prediction-boundary")).toHaveCSS("stroke-width", "1px");
    await expect(page.locator(".minecraft-prediction-boundary-casing")).toHaveCount(1);
    await expect(page.locator(".minecraft-prediction-cell").first()).toHaveCSS("stroke", "none");
    await expect(page.getByText(/Biome boundaries may differ from generated terrain/)).toBeVisible();
    expect(fixture.predictions).toEqual([{ world: "minecraft:overworld", min_x: 64, min_z: 64, y: -16 }]);
    await page.getByText("Predicted · Y -16 · Large biomes", { exact: true }).scrollIntoViewIfNeeded();
    await page.screenshot({ path: `../target/minecraft-tests/prediction-${width}.png` });
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(width);
    await page.locator(".minecraft-atlas-canvas").click({ position: { x: 140, y: 130 } });
    expect(fixture.predictions).toHaveLength(1);
    await page.clock.fastForward(16000);
    await expect(page.locator(".minecraft-prediction-cell")).toHaveCount(0);
    await expect(page.locator(".minecraft-prediction-boundary")).toHaveCount(0);
    await expect(page.getByText(/Preview expired\. Request a fresh coverage/)).toBeVisible();
    expect(fixture.predictions).toHaveLength(1);
    expect(errors).toEqual([]);
  });
}

test("actual observations outrank predictions and terrain refresh clears the snapshot", async ({ page }) => {
  await installApiMocks(page, "logged-out"); await setUiPreferences(page, "en-US", "light");
  await installMinecraftMapMocks(page);
  await page.goto("/minecraft");
  await page.getByRole("button", { name: "Preview selected area" }).click();
  await expect(page.locator(".minecraft-prediction-cell")).toHaveCount(976);
  await page.getByRole("button", { name: "Survey selected area" }).click();
  await expect(page.locator(".minecraft-prediction-cell")).toHaveCount(64);
  await expect(page.locator(".minecraft-prediction-boundary")).toHaveCount(1);
  await expect(page.getByText(/64 predicted cells/)).toBeVisible();
  await page.getByRole("button", { name: "Refresh terrain" }).click();
  await expect(page.locator(".minecraft-prediction-cell")).toHaveCount(0);
  await expect(page.locator(".minecraft-prediction-boundary")).toHaveCount(0);
  await expect(page.locator(".minecraft-prediction-receipt")).toHaveCount(0);
  await expect(page.getByText("Terrain refreshed. Request a fresh prediction preview.")).toBeVisible();
});
