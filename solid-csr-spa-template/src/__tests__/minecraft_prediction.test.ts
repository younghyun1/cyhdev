import { describe, expect, it } from "vitest";
import type { MinecraftMapData, MinecraftPrediction } from "../generated";
import { predictionBoundary, predictionCells, predictionEdgeCells } from "../components/minecraft/predictionMask";

function prediction(): MinecraftPrediction {
  return {
    world: "minecraft:overworld", sampled_at_ms: 1000, expires_at_ms: 16000, generator_revision: "fixture", preset: "large_biomes", y: 64, step: 4, min_x: -64, min_z: -64,
    coverage: Array.from({ length: 64 }, (_, i) => ({ chunk_x: i % 8 - 4, chunk_z: Math.floor(i / 8) - 4, state: "ungenerated" })),
    cells: Array.from({ length: 1024 }, (_, i) => ({ x: i % 32 * 4 - 64, z: Math.floor(i / 32) * 4 - 64, biome: "minecraft:plains" })),
  };
}

describe("predicted biome coverage", () => {
  it("admits only confirmed ungenerated chunks, including negative coordinates", () => {
    const base = prediction();
    const result: MinecraftPrediction = { ...base, coverage: base.coverage.map((chunk, i) => ({ ...chunk, state: i === 0 ? "generated" : i === 1 ? "unknown" : i === 2 ? "excluded" : chunk.state })) };
    const cells = predictionCells(result, []);
    expect(cells).toHaveLength(976);
    expect(cells.some(cell => cell.z < -48 && cell.x < -16)).toBe(false);
    expect(cells.some(cell => cell.x === -4 && cell.z === -4)).toBe(true);
  });

  it("fails closed on missing coverage, duplicate entries and invalid grids", () => {
    const result = prediction();
    expect(predictionCells({ ...result, coverage: result.coverage.slice(1) }, [])).toEqual([]);
    expect(predictionCells({ ...result, step: 16 }, [])).toEqual([]);
    const duplicate = { ...result, coverage: result.coverage.map((chunk, i) => i === 1 ? result.coverage[0]! : chunk) };
    expect(predictionCells(duplicate, []).filter(cell => cell.z < -48 && cell.x < -32)).toEqual([]);
    const invalid = { ...result, cells: [{ x: -63, z: -64, biome: "minecraft:plains" }, { x: 64, z: 0, biome: "minecraft:plains" }] };
    expect(predictionCells(invalid, [])).toEqual([]);
  });

  it("lets actual survey and block evidence suppress an entire predicted chunk", () => {
    const observed: MinecraftMapData = { kind: "area", world: "minecraft:overworld", sampled_at_ms: 2000, scanned_chunks: 1, missing_chunks: 0, truncated: false, worlds: [], blocks: [], cells: [{ x: -4, y: 64, z: -4, biome: "minecraft:forest" }], matches: [{ x: 0, y: 20, z: 0 }], structures: [] };
    expect(predictionCells(prediction(), [observed])).toHaveLength(992);
    expect(predictionCells(prediction(), [{ ...observed, world: "minecraft:the_nether" }])).toHaveLength(1024);
  });

  it("outlines only the predicted perimeter without shared chunk edges", () => {
    const cells = predictionCells(prediction(), []);
    const outline = predictionBoundary(cells);
    expect(outline).toHaveLength(32);
    expect(outline.every(([a, b]) => a.x === b.x && Math.abs(a.x) === 64 || a.z === b.z && Math.abs(a.z) === 64)).toBe(true);
    const withHole = cells.filter(cell => Math.floor(cell.x / 16) !== 0 || Math.floor(cell.z / 16) !== 0);
    const holeOutline = predictionBoundary(withHole);
    expect(holeOutline).toHaveLength(36);
    const holeEdges = holeOutline.filter(edge => edge.every(point => point.x >= 0 && point.x <= 16 && point.z >= 0 && point.z <= 16));
    expect(holeEdges).toHaveLength(4);
    expect(predictionBoundary([])).toEqual([]);
  });

  it("feathers only cells inside the confirmed predicted perimeter", () => {
    const cells = predictionCells(prediction(), []);
    const edges = predictionEdgeCells(cells);
    expect(edges.size).toBe(124);
    expect(edges.has("-64,-64")).toBe(true);
    expect(edges.has("-4,-4")).toBe(false);
    expect(edges.has("64,0")).toBe(false);
    expect([...edges].every(key => cells.some(cell => key === `${cell.x},${cell.z}`))).toBe(true);
  });
});
