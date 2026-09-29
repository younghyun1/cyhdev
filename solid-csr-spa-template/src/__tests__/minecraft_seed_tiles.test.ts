import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { MinecraftSeedTile, MinecraftSeedTileQuery } from "../generated";
import { ApiContractError } from "../generated";
import { createSeedTiles, decodeSeedTile, seedTileOrigin, type SeedTiles } from "../components/minecraft/seedTiles";
import { alphaBoundary, decodeAlphaEdge } from "../components/minecraft/seedTileLayer";

const query: MinecraftSeedTileQuery = { world: "minecraft:overworld", tile_x: -1, tile_z: 0, level: 0, y: 64 };
function reply(q = query, change: Partial<MinecraftSeedTile> = {}): MinecraftSeedTile {
  const origin = seedTileOrigin(q);
  return { ...q, min_x: origin.x, min_z: origin.z, width: 64, height: 64, step: 4 * 2 ** q.level, palette: ["minecraft:plains", "minecraft:forest"], indices: Array.from({ length: 4096 }, (_, i) => i % 2), preset: "large_biomes", profile_epoch: "a".repeat(64), generator_revision: "fixture", sampled_at_ms: Date.now(), expires_at_ms: Date.now() + 15000, ...change };
}
function deferred<T>() { let resolve!: (value: T) => void; const promise = new Promise<T>(done => { resolve = done; }); return { promise, resolve }; }
const stores: SeedTiles[] = [];
function setup() {
  const read = vi.fn(async (q: MinecraftSeedTileQuery, _signal: AbortSignal) => reply(q));
  const changed = vi.fn(), updated = vi.fn();
  const store = createSeedTiles({ read, changed, updated }); stores.push(store);
  store.configure(query.world, query.y, true);
  return { store, read, changed, updated };
}
const tick = (ms = 0) => vi.advanceTimersByTimeAsync(ms);

describe("continuous seed tile cache", () => {
  beforeEach(() => { vi.useFakeTimers(); vi.setSystemTime(1_000_000); });
  afterEach(() => { for (const store of stores.splice(0)) store.dispose(); vi.restoreAllMocks(); vi.useRealTimers(); });

  it("uses globally aligned negative tiles and level-dependent sample resolution", () => {
    expect(seedTileOrigin({ tile_x: -1, tile_z: -2, level: 3 })).toEqual({ x: -2048, z: -4096 });
    expect(decodeSeedTile(reply(), query)?.indices).toBeInstanceOf(Uint16Array);
    for (const change of [{ world: "minecraft:the_nether" }, { step: 8 }, { min_x: -255 }, { width: 32 }, { profile_epoch: "" }, { indices: [0] }, { indices: Array(4096).fill(2) as number[] }, { expires_at_ms: Date.now() }, { expires_at_ms: Date.now() + 16000 }, { sampled_at_ms: Date.now() - 16000 }, { sampled_at_ms: Date.now() - 1000 }]) expect(decodeSeedTile(reply(query, change), query)).toBeNull();
  });

  it("retains current tiles while adjacent requests arrive and reuses warm data", async () => {
    const { store, read } = setup(), first = vi.fn(), next = vi.fn();
    const release = store.subscribe(query, first, () => 0); await tick();
    expect(store.sample(query.world, { x: -1, z: 1 }, 64)).toMatchObject({ sample: { x: -4, z: 0 }, step: 4, name: "minecraft:forest" });
    const hold = deferred<MinecraftSeedTile>(); read.mockReturnValueOnce(hold.promise);
    store.subscribe({ ...query, tile_x: 0 }, next, () => 1); await tick();
    expect(first.mock.lastCall?.[0]).not.toBeNull();
    release(); const reused = vi.fn(); store.subscribe(query, reused, () => 0);
    expect(read).toHaveBeenCalledTimes(2); expect(reused.mock.lastCall?.[0]).not.toBeNull();
    hold.resolve(reply({ ...query, tile_x: 0 })); await tick();
  });

  it("limits requests to four and drops obsolete queued viewport work", async () => {
    const { store, read } = setup(); const held = deferred<MinecraftSeedTile>(); read.mockReturnValue(held.promise);
    const release: (() => void)[] = [];
    for (let i = 0; i < 300; ++i) release.push(store.subscribe({ ...query, tile_x: i }, () => undefined, () => i));
    expect(read).toHaveBeenCalledTimes(4); expect(store.stats().active).toBe(256);
    for (const unsubscribe of release) unsubscribe();
    expect(read.mock.calls.every(([, signal]) => signal.aborted)).toBe(true);
    held.resolve(reply({ ...query, tile_x: 0 })); await tick();
    expect(read).toHaveBeenCalledTimes(4); expect(store.stats().entries).toBe(0);
  });

  it("refreshes before expiry without flashing and clears expired permission on failure", async () => {
    const { store, read } = setup(), draw = vi.fn(); store.subscribe(query, draw, () => 0); await tick();
    read.mockRejectedValue(new Error("Unavailable"));
    await tick(12000); expect(read).toHaveBeenCalledTimes(2); expect(draw.mock.lastCall?.[0]).not.toBeNull();
    await tick(3000); expect(draw.mock.lastCall?.[0]).toBeNull();
    expect(store.sample(query.world, { x: -4, z: 0 }, 64)).toBeNull();
    read.mockImplementation(async q => reply(q)); await tick(6000);
    expect(draw.mock.lastCall?.[0]).not.toBeNull();
  });

  it("expires permission at its exact deadline between polling ticks", async () => {
    const { store, read } = setup(), draw = vi.fn();
    read.mockImplementationOnce(async q => reply(q, { sampled_at_ms: Date.now() - 137, expires_at_ms: Date.now() + 14_863 }));
    store.subscribe(query, draw, () => 0); await tick(); read.mockRejectedValue(new Error("Unavailable"));
    await tick(14_862); expect(draw.mock.lastCall?.[0]).not.toBeNull();
    await tick(1); expect(draw.mock.lastCall?.[0]).toBeNull();
  });

  it("keeps null palette indices hidden and selects the finest cached level", async () => {
    const { store, read } = setup(); const coarse = { ...query, level: 1 }; store.subscribe(coarse, () => undefined, () => 1); await tick();
    expect(store.sample(query.world, { x: -1, z: 1 }, 64)?.step).toBe(8);
    read.mockImplementation(async q => reply(q, { indices: Array(4096).fill(null) as null[] }));
    store.subscribe(query, () => undefined, () => 0); await tick();
    expect(store.sample(query.world, { x: -1, z: 1 }, 64)).toBeNull();
  });

  it.each(["disabled", "dimension", "height", "disposed"] as const)("rejects late replies after %s changes", async reason => {
    const { store, read } = setup(), hold = deferred<MinecraftSeedTile>(), draw = vi.fn(); read.mockReturnValue(hold.promise);
    store.subscribe(query, draw, () => 0);
    if (reason === "disposed") store.dispose();
    else store.configure(reason === "dimension" ? "minecraft:the_nether" : query.world, reason === "height" ? -16 : 64, reason !== "disabled");
    hold.resolve(reply()); await tick();
    expect(draw.mock.lastCall?.[0]).toBeNull(); expect(store.stats().entries).toBe(0);
  });

  it("retries overload with backoff while permanent client errors remain stopped", async () => {
    const { store, read } = setup(); read.mockRejectedValueOnce(new ApiContractError(429, "Busy"));
    store.subscribe(query, () => undefined, () => 0); await tick(1999); expect(read).toHaveBeenCalledTimes(1);
    await tick(1); expect(read).toHaveBeenCalledTimes(2);
    read.mockRejectedValue(new ApiContractError(400, "Invalid"));
    store.subscribe({ ...query, tile_x: 0 }, () => undefined, () => 0); await tick(3000);
    expect(read).toHaveBeenCalledTimes(3);
  });

  it("bounds cache allocations across long pans", async () => {
    const { store, read } = setup();
    const palette = Array.from({ length: 256 }, (_, i) => `minecraft:${String(i).padStart(110, "a")}`);
    read.mockImplementation(async q => reply(q, { palette }));
    for (let i = 0; i < 420; ++i) { const release = store.subscribe({ ...query, tile_x: i }, () => undefined, () => 0); await tick(); release(); }
    expect(store.stats().bytes).toBeLessThanOrEqual(16 * 1024 * 1024);
    expect(store.stats().entries).toBeLessThan(420);
  });

  it("rejects a different profile at the same timestamp and clears older tiles for a newer profile", async () => {
    const { store, read } = setup(), first = vi.fn(); store.subscribe(query, first, () => 0); await tick();
    read.mockImplementation(async q => reply(q, { profile_epoch: "b".repeat(64) }));
    const second = vi.fn(); store.subscribe({ ...query, tile_x: 0 }, second, () => 0); await tick();
    expect(second.mock.lastCall?.[0]).toBeNull(); expect(first.mock.lastCall?.[0]).not.toBeNull();
    await tick(2000);
    expect(second.mock.lastCall?.[0]?.profile_epoch).toBe("b".repeat(64));
    expect(first.mock.calls.some(([tile]) => tile === null)).toBe(true);
    expect(store.sample(query.world, { x: -1, z: 0 }, 64)).not.toBeNull();
  });
});

describe("rendered terrain frontier", () => {
  it("does not draw tile seams or unknown-neighbor outlines", () => {
    const opaque = new Uint8Array(16).fill(255);
    expect(alphaBoundary(opaque, () => 255, () => true, 4)).toHaveLength(0);
    expect(alphaBoundary(opaque, () => null, () => true, 4)).toHaveLength(0);
    expect(alphaBoundary(opaque, () => 0, () => false, 4)).toHaveLength(0);
  });
  it("draws only actual/predicted transitions including a frontier across a tile edge", () => {
    const opaque = new Uint8Array(16).fill(255);
    const edges = alphaBoundary(opaque, x => x === 4 ? 0 : 255, () => true, 4);
    expect(edges).toHaveLength(4); expect([...edges].map(edge => decodeAlphaEdge(edge, 4)).every(edge => edge.x === 3 && edge.dx === 1)).toBe(true);
    const half = Uint8Array.from([255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0]);
    expect(alphaBoundary(half, () => null, () => true, 4)).toHaveLength(4);
  });
  it("caps complex alpha geometry before allocating unbounded edge objects", () => {
    const noise = Uint8Array.from({ length: 256 * 256 }, (_, i) => (i + Math.floor(i / 256)) % 2 ? 255 : 0);
    expect(alphaBoundary(noise, () => 0, () => true)).toHaveLength(4096);
  });
});
