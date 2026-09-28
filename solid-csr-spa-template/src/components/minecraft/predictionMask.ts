import type { MinecraftMapData, MinecraftPredictedBiome, MinecraftPrediction } from "../../generated";
import type { MapPoint } from "./mapMath";

export type PredictionEdge = readonly [MapPoint, MapPoint];
const CELL_NEIGHBORS = [[-4, 0], [4, 0], [0, -4], [0, 4]] as const;

/** Shared chunk edges disappear, leaving only the outside perimeter and coverage holes. */
export function predictionBoundary(cells: readonly MinecraftPredictedBiome[]): PredictionEdge[] {
  const chunks = new Map<string, MapPoint>();
  for (const cell of cells) {
    const chunk = { x: Math.floor(cell.x / 16), z: Math.floor(cell.z / 16) };
    chunks.set(`${chunk.x},${chunk.z}`, chunk);
  }
  const edges: PredictionEdge[] = [];
  for (const chunk of chunks.values()) {
    const x = chunk.x * 16, z = chunk.z * 16;
    if (!chunks.has(`${chunk.x},${chunk.z - 1}`)) edges.push([{ x, z }, { x: x + 16, z }]);
    if (!chunks.has(`${chunk.x + 1},${chunk.z}`)) edges.push([{ x: x + 16, z }, { x: x + 16, z: z + 16 }]);
    if (!chunks.has(`${chunk.x},${chunk.z + 1}`)) edges.push([{ x: x + 16, z: z + 16 }, { x, z: z + 16 }]);
    if (!chunks.has(`${chunk.x - 1},${chunk.z}`)) edges.push([{ x, z: z + 16 }, { x, z }]);
  }
  return edges;
}

/** Fade only existing edge cells; no color is extended into an unconfirmed chunk. */
export function predictionEdgeCells(cells: readonly MinecraftPredictedBiome[]): ReadonlySet<string> {
  const present = new Set(cells.map(cell => `${cell.x},${cell.z}`));
  return new Set(cells.filter(cell => CELL_NEIGHBORS.some(([x, z]) => !present.has(`${cell.x + x},${cell.z + z}`))).map(cell => `${cell.x},${cell.z}`));
}

/** A missing or conflicting coverage entry never permits a predicted cell. */
export function predictionCells(prediction: MinecraftPrediction, observations: readonly (MinecraftMapData | null)[]): MinecraftPredictedBiome[] {
  if (prediction.step !== 4 || prediction.cells.length > 1024 || prediction.coverage.length !== 64) return [];
  const coverage = new Map<string, boolean>();
  for (const chunk of prediction.coverage) {
    const key = `${chunk.chunk_x},${chunk.chunk_z}`;
    coverage.set(key, !coverage.has(key) && chunk.state === "ungenerated");
  }
  for (const observation of observations) {
    if (observation?.world !== prediction.world) continue;
    for (const cell of [...observation.cells, ...observation.matches]) coverage.set(`${Math.floor(cell.x / 16)},${Math.floor(cell.z / 16)}`, false);
  }
  return prediction.cells.filter(cell => Number.isInteger(cell.x) && Number.isInteger(cell.z)
    && cell.x % 4 === 0 && cell.z % 4 === 0
    && cell.x >= prediction.min_x && cell.x < prediction.min_x + 128
    && cell.z >= prediction.min_z && cell.z < prediction.min_z + 128
    && coverage.get(`${Math.floor(cell.x / 16)},${Math.floor(cell.z / 16)}`) === true);
}
