import type { MapPoint } from "../components/minecraft/mapMath";

export type SquaremapWorld = { readonly name: string; readonly displayName: string; readonly type: string };
export type SquaremapSettings = { readonly maxZoom: number; readonly defaultZoom: number; readonly extraZoom: number; readonly spawn: MapPoint };
export type SquaremapPlayer = MapPoint & { readonly name: string; readonly world: string };
const MAX_JSON_BYTES = 1_048_576;

function record(value: unknown): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) throw new Error("Invalid map metadata.");
  return value as Record<string, unknown>;
}
function text(value: unknown, limit: number): string {
  if (typeof value !== "string" || value.length > limit) throw new Error("Invalid map metadata text.");
  return value;
}
function number(value: unknown, min: number, max: number): number {
  if (typeof value !== "number" || !Number.isFinite(value) || value < min || value > max) throw new Error("Invalid map metadata coordinates.");
  return value;
}
export function mapId(value: unknown): string {
  const name = text(value, 128);
  if (!/^[a-zA-Z0-9_-][a-zA-Z0-9_.-]*$/.test(name) || name.includes("..")) throw new Error("Invalid map world path.");
  return name;
}
export function parseWorlds(value: unknown): SquaremapWorld[] {
  const worlds = record(value).worlds;
  if (!Array.isArray(worlds) || worlds.length > 64) throw new Error("Invalid map world list.");
  return worlds.map((entry: unknown) => {
    const world = record(entry);
    return { name: mapId(world.name), displayName: text(world.display_name, 160), type: text(world.type, 64) };
  });
}
export function parseSettings(value: unknown): SquaremapSettings {
  const settings = record(value), zoom = record(settings.zoom), spawn = record(settings.spawn);
  const maxZoom = number(zoom.max, 0, 12), extraZoom = number(zoom.extra, 0, 8);
  const defaultZoom = number(zoom.def, 0, maxZoom + extraZoom);
  if (![maxZoom, extraZoom, defaultZoom].every(Number.isInteger)) throw new Error("Invalid map zoom.");
  return { maxZoom, defaultZoom, extraZoom, spawn: { x: number(spawn.x, -30_000_000, 30_000_000), z: number(spawn.z, -30_000_000, 30_000_000) } };
}
export function parsePlayers(value: unknown): SquaremapPlayer[] {
  const players = record(value).players;
  if (!Array.isArray(players) || players.length > 1000) throw new Error("Invalid map player list.");
  return players.flatMap((entry: unknown) => {
    const player = record(entry);
    if (player.x === undefined || player.z === undefined) return [];
    return [{ name: text(player.name, 80), world: mapId(player.world), x: number(player.x, -30_000_000, 30_000_000), z: number(player.z, -30_000_000, 30_000_000) }];
  });
}

/** Server-generated map JSON is untrusted; bound the stream before parsing it. */
async function readJson(path: string, signal: AbortSignal): Promise<unknown> {
  const response = await fetch(`/minecraft/map/tiles/${path}`, { signal, cache: "no-store", credentials: "omit" });
  if (!response.ok || !response.body) throw new Error("Map data is unavailable. The original map is still available below.");
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let size = 0, content = "";
  try {
    for (;;) {
      const next = await reader.read();
      if (next.done) break;
      size += next.value.byteLength;
      if (size > MAX_JSON_BYTES) throw new Error("Map metadata exceeds its size limit.");
      content += decoder.decode(next.value, { stream: true });
    }
    content += decoder.decode();
    return JSON.parse(content) as unknown;
  } finally {
    await reader.cancel();
    reader.releaseLock();
  }
}

export const squaremap = {
  worlds: async (signal: AbortSignal) => parseWorlds(await readJson("settings.json", signal)),
  settings: async (world: string, signal: AbortSignal) => parseSettings(await readJson(`${mapId(world)}/settings.json`, signal)),
  players: async (signal: AbortSignal) => parsePlayers(await readJson("players.json", signal)),
};
