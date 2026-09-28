export type MapPoint = { readonly x: number; readonly z: number };
export const WORLD_LIMIT = 30_000_000;

export function coordinate(value: string): number | null {
  if (!/^-?\d+$/.test(value.trim())) return null;
  const result = Number(value);
  return Number.isSafeInteger(result) && Math.abs(result) <= WORLD_LIMIT ? result : null;
}

/** Floor division preserves chunk boundaries west and north of the origin. */
export function scanOrigin(point: MapPoint, width: number) {
  const margin = Math.floor(width / 2);
  return { chunk_x: Math.max(-1_875_000, Math.min(1_875_000 - width, Math.floor(point.x / 16) - margin)), chunk_z: Math.max(-1_875_000, Math.min(1_875_000 - width, Math.floor(point.z / 16) - margin)), width, height: width };
}

export function distance(a: MapPoint, b: MapPoint): number {
  return Math.hypot(b.x - a.x, b.z - a.z);
}

export function biomeColor(biome: string): string {
  let hash = 0;
  for (const character of biome) hash = (hash * 31 + character.charCodeAt(0)) >>> 0;
  return `hsl(${hash % 360} 62% 55%)`;
}

export function displayName(value: string): string {
  return value.replace(/^minecraft:/, "").replaceAll("_", " ");
}

export function elevationColor(y: number, min: number, max: number): string {
  const fraction = Math.max(0, Math.min(1, (y - min) / Math.max(1, max - min)));
  return `hsl(${130 - fraction * 110} 65% ${30 + fraction * 45}%)`;
}
