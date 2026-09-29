import { createRoot, flush } from "solid-js";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createMapExplorer } from "../components/minecraft/createMapExplorer";
import { ApiContractError, type MinecraftMapData, type MinecraftPrediction } from "../generated";

const transport = vi.hoisted(() => ({ query: vi.fn(), prediction: vi.fn(), worlds: vi.fn(), settings: vi.fn(), players: vi.fn() }));
vi.mock("../services/account_api", () => ({ contractApi: { minecraftMapQuery: transport.query, minecraftMapPrediction: transport.prediction } }));
vi.mock("../services/squaremap", () => ({ squaremap: { worlds: transport.worlds, settings: transport.settings, players: transport.players } }));
const worlds = [
  { id: "minecraft:overworld", name: "Overworld", map_id: "minecraft_overworld", min_y: -64, max_y: 319 },
  { id: "minecraft:the_nether", name: "Nether", map_id: "minecraft_the_nether", min_y: 0, max_y: 255 },
];
const catalog: MinecraftMapData = { kind: "catalog", world: null, sampled_at_ms: 1, scanned_chunks: 0, missing_chunks: 0, truncated: false, worlds, blocks: [], cells: [], structures: [], matches: [] };
function reply(changes: Partial<MinecraftPrediction> = {}): { data: MinecraftPrediction } {
  return { data: { world: worlds[0]!.id, sampled_at_ms: Date.now(), expires_at_ms: Date.now() + 15_000, generator_revision: "fixture", preset: "default", y: 64, min_x: -64, min_z: -64, step: 4, coverage: [], cells: [], ...changes } };
}
const disposals: (() => void)[] = [];
async function tick(ms = 0) { await vi.advanceTimersByTimeAsync(ms); flush(); }
async function start() {
  const explorer = createRoot(dispose => { disposals.push(dispose); return createMapExplorer(); });
  flush(); await tick(); await tick(1000);
  expect(transport.prediction).toHaveBeenCalledTimes(1);
  return explorer;
}

describe("Minecraft automatic prediction recovery", () => {
  beforeEach(() => {
    vi.useFakeTimers(); vi.setSystemTime(1_000_000);
    for (const mock of Object.values(transport)) mock.mockReset();
    transport.worlds.mockResolvedValue([{ name: "minecraft_overworld", displayName: "Overworld", type: "normal" }, { name: "minecraft_the_nether", displayName: "Nether", type: "nether" }]);
    transport.settings.mockResolvedValue({ maxZoom: 3, defaultZoom: 3, extraZoom: 2, spawn: { x: 0, z: 0 } });
    transport.players.mockResolvedValue([]); transport.query.mockResolvedValue({ data: catalog });
    transport.prediction.mockImplementation(() => Promise.resolve(reply()));
  });
  afterEach(() => { while (disposals.length) disposals.pop()?.(); vi.restoreAllMocks(); vi.useRealTimers(); });

  it("recovers a busy response without any terrain-refresh event", async () => {
    transport.prediction.mockRejectedValueOnce(new ApiContractError(429, '{"error_code":87}'));
    const explorer = await start();
    await tick(1999); expect(transport.prediction).toHaveBeenCalledTimes(1);
    await tick(1); expect(transport.prediction).toHaveBeenCalledTimes(2);
    expect(explorer.prediction()?.world).toBe(worlds[0]!.id);
    expect(explorer.predictionNotice()).toBe("");
  });

  it("caps consecutive network failures at three retries with increasing delay", async () => {
    transport.prediction.mockRejectedValue(new TypeError("Network unavailable"));
    const explorer = await start();
    for (const [index, delay] of [2000, 5000, 10_000].entries()) {
      await tick(delay - 1); expect(transport.prediction).toHaveBeenCalledTimes(index + 1);
      await tick(1); expect(transport.prediction).toHaveBeenCalledTimes(index + 2);
    }
    await tick(60_000); expect(transport.prediction).toHaveBeenCalledTimes(4);
    expect(explorer.predicting()).toBe(false);
    expect(explorer.predictionNotice()).toBe("Network unavailable");
  });

  it.each([86, 89])("retries ambiguous unavailable errors but not disabled integration (%i)", async code => {
    transport.prediction.mockRejectedValue(new ApiContractError(503, JSON.stringify({ error_code: code })));
    await start(); await tick(20_000);
    expect(transport.prediction).toHaveBeenCalledTimes(code === 86 ? 1 : 4);
  });

  it("does not retry permanent client errors or mismatched-world payloads", async () => {
    transport.prediction.mockRejectedValueOnce(new ApiContractError(400, "Invalid request"));
    const explorer = await start(); await tick(20_000);
    expect(transport.prediction).toHaveBeenCalledTimes(1);
    transport.prediction.mockImplementation(() => Promise.resolve(reply({ world: worlds[1]!.id })));
    explorer.terrainRefreshing(); await tick(700); await tick(20_000);
    expect(transport.prediction).toHaveBeenCalledTimes(2); expect(explorer.prediction()).toBeNull();
  });

  it("retries an expired snapshot but accepts only a fresh response", async () => {
    transport.prediction.mockResolvedValueOnce(reply({ expires_at_ms: Date.now() - 1 }));
    const explorer = await start(); expect(explorer.prediction()).toBeNull();
    await tick(2000); expect(transport.prediction).toHaveBeenCalledTimes(2);
    expect(explorer.prediction()?.expires_at_ms).toBeGreaterThan(Date.now());
  });

  it.each(["disabled", "world", "hidden", "disposed"] as const)("cancels pending retries when %s", async change => {
    transport.prediction.mockRejectedValue(new TypeError("Network unavailable"));
    const explorer = await start();
    if (change === "disabled") explorer.setPredictionsEnabled(false);
    if (change === "world") await explorer.selectWorld("minecraft_the_nether");
    if (change === "hidden") { vi.spyOn(document, "visibilityState", "get").mockReturnValue("hidden"); document.dispatchEvent(new Event("visibilitychange")); }
    if (change === "disposed") disposals.pop()?.();
    flush(); await tick(20_000); expect(transport.prediction).toHaveBeenCalledTimes(1);
  });
});
