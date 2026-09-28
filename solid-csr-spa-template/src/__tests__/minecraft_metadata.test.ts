import { describe, expect, it } from "vitest";
import { coordinate, distance, scanOrigin } from "../components/minecraft/mapMath";
import { mapId, parsePlayers, parseSettings, parseWorlds } from "../services/squaremap";

describe("Minecraft metadata boundaries", () => {
  it("rejects paths and oversized world lists from plugin metadata", () => {
    for (const value of ["../private", "a/b", "https://evil.test", "a\\b", "..", "a?script=1"]) expect(() => mapId(value)).toThrow();
    expect(() => parseWorlds({ worlds: Array.from({ length: 65 }, () => ({})) })).toThrow();
    expect(mapId("minecraft_the_nether")).toBe("minecraft_the_nether");
  });
  it("bounds zoom and treats display names as plain text", () => {
    expect(parseWorlds({ worlds: [{ name: "world", display_name: "<script>hello</script>", type: "normal" }] })[0]?.displayName).toBe("<script>hello</script>");
    expect(() => parseSettings({ zoom: { max: 30, def: 1, extra: 0 }, spawn: { x: 0, z: 0 } })).toThrow();
    expect(() => parseSettings({ zoom: { max: 3, def: 1, extra: 0 }, spawn: { x: Infinity, z: 0 } })).toThrow();
  });
  it("omits players without published coordinates and bounds player data", () => {
    expect(parsePlayers({ players: [{ name: "Hidden", world: "world" }, { name: "Alex", world: "world", x: -1, z: 2 }] })).toEqual([{ name: "Alex", world: "world", x: -1, z: 2 }]);
    expect(() => parsePlayers({ players: [{ name: "Alex", world: "world", x: 99_000_000, z: 2 }] })).toThrow();
  });
  it("uses floor division at negative coordinates and bounds edge scans", () => {
    expect(scanOrigin({ x: -1, z: -16 }, 8)).toEqual({ chunk_x: -5, chunk_z: -5, width: 8, height: 8 });
    expect(scanOrigin({ x: 30_000_000, z: -30_000_000 }, 4)).toEqual({ chunk_x: 1_874_996, chunk_z: -1_875_000, width: 4, height: 4 });
    expect(coordinate("1e4")).toBeNull(); expect(coordinate("NaN")).toBeNull(); expect(coordinate("30000001")).toBeNull(); expect(coordinate("-12")).toBe(-12);
    expect(distance({ x: 0, z: 0 }, { x: 3, z: 4 })).toBe(5);
  });
});
