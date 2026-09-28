import { createSignal, onCleanup, onSettled } from "solid-js";
import type { MinecraftMapData, MinecraftMapQuery } from "../../generated";
import { contractApi } from "../../services/account_api";
import { squaremap, type SquaremapPlayer, type SquaremapSettings, type SquaremapWorld } from "../../services/squaremap";
import { coordinate, scanOrigin, type MapPoint } from "./mapMath";

export type LoadedMap = { readonly world: SquaremapWorld; readonly settings: SquaremapSettings };
export const mapFailure = (cause: unknown) => cause instanceof Error ? cause.message : "The map request failed.";

/** World changes invalidate pending results; game scans only follow explicit actions. */
export function createMapExplorer() {
  const [worlds, setWorlds] = createSignal<SquaremapWorld[]>([]);
  const [loaded, setLoaded] = createSignal<LoadedMap | null>(null);
  const [catalog, setCatalog] = createSignal<MinecraftMapData | null>(null);
  const [area, setArea] = createSignal<MinecraftMapData | null>(null);
  const [matches, setMatches] = createSignal<MinecraftMapData | null>(null);
  const [areaRegion, setAreaRegion] = createSignal<ReturnType<typeof scanOrigin> | null>(null);
  const [matchRegion, setMatchRegion] = createSignal<ReturnType<typeof scanOrigin> | null>(null);
  const [areaSlice, setAreaSlice] = createSignal<number | null>(null);
  const [matchedBlock, setMatchedBlock] = createSignal("");
  const [players, setPlayers] = createSignal<SquaremapPlayer[]>([]);
  const [point, setPoint] = createSignal<MapPoint>({ x: 0, z: 0 });
  const [view, setView] = createSignal<MapPoint>({ x: 0, z: 0 });
  const [busy, setBusy] = createSignal(false);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal("");
  const [catalogError, setCatalogError] = createSignal("");
  const [playerError, setPlayerError] = createSignal("");
  const [refresh, setRefresh] = createSignal(0);
  let active = true, visible = true, epoch = 0, queryPending = false, metadataPending = false, playersPending = false;
  const lifetime = new AbortController();
  let worldRequest: AbortController | null = null;
  let queryRequest: AbortController | null = null;
  const currentWorld = () => catalog()?.worlds.find(world => world.map_id === loaded()?.world.name);

  const refreshPlayers = async () => {
    if (playersPending || !active || !visible) return;
    playersPending = true;
    try {
      const result = await squaremap.players(lifetime.signal);
      if (active) { setPlayers(result); setPlayerError(""); }
    } catch {
      if (active) { setPlayers([]); setPlayerError("Player positions are unavailable."); }
    } finally { playersPending = false; }
  };

  const selectWorld = async (name: string, initialPoint?: MapPoint, candidates = worlds()) => {
    const world = candidates.find(item => item.name === name);
    if (!world) return;
    const revision = ++epoch;
    worldRequest?.abort(); queryRequest?.abort();
    const request = new AbortController(); worldRequest = request;
    setLoaded(null); setArea(null); setMatches(null); setAreaRegion(null); setMatchRegion(null); setLoading(true); setError("");
    try {
      const settings = await squaremap.settings(world.name, request.signal);
      if (!active || revision !== epoch) return;
      const center = initialPoint ?? settings.spawn;
      setPoint(center); setView(center); setLoaded({ world, settings });
    } catch (cause: unknown) {
      if (active && revision === epoch) setError(mapFailure(cause));
    } finally { if (active && revision === epoch) setLoading(false); }
  };

  const loadCatalog = async () => {
    if (queryPending) return;
    queryPending = true; setBusy(true); setCatalogError("");
    try {
      const response = await contractApi.minecraftMapQuery({ body: { kind: "catalog" } }, { signal: lifetime.signal });
      if (active) setCatalog(response.data);
    } catch {
      if (active) { setCatalog(null); setCatalogError("World analysis is unavailable. Terrain and coordinate tools remain available."); }
    } finally { queryPending = false; if (active) setBusy(false); }
  };
  const initialize = async () => {
    if (metadataPending) return;
    metadataPending = true; setLoading(true); setError("");
    void loadCatalog();
    try {
      const available = await squaremap.worlds(lifetime.signal);
      if (!active) return;
      setWorlds(available);
      const params = new URLSearchParams(window.location.search);
      const initialWorld = available.find(world => world.name === params.get("world")) ?? available[0];
      const x = coordinate(params.get("x") ?? ""), z = coordinate(params.get("z") ?? "");
      if (initialWorld) await selectWorld(initialWorld.name, x === null || z === null ? undefined : { x, z }, available);
      else { setLoading(false); setError("No rendered worlds are available yet."); }
      void refreshPlayers();
    } catch (cause: unknown) {
      if (active) { setLoading(false); setError(mapFailure(cause)); }
    } finally { metadataPending = false; }
  };

  const query = async (body: MinecraftMapQuery) => {
    if (queryPending || !active) return;
    queryPending = true; setBusy(true); setError("");
    const revision = epoch, request = new AbortController(); queryRequest = request;
    try {
      const response = await contractApi.minecraftMapQuery({ body }, { signal: request.signal });
      if (!active || revision !== epoch) return;
      if (body.kind === "area") { setArea(response.data); setAreaRegion(body); setAreaSlice(body.y ?? null); }
      else if (body.kind === "blocks") { setMatches(response.data); setMatchRegion(body); setMatchedBlock(body.block); }
    } catch (cause: unknown) {
      if (active && revision === epoch) setError(mapFailure(cause));
    } finally { queryPending = false; if (active) setBusy(false); }
  };

  const scan = (y: number | null) => {
    const world = currentWorld();
    if (world) void query({ kind: "area", world: world.id, ...scanOrigin(point(), 8), y });
  };
  const searchBlocks = (block: string, min_y: number, max_y: number) => {
    const world = currentWorld();
    if (world) void query({ kind: "blocks", world: world.id, ...scanOrigin(point(), 4), block, min_y, max_y });
  };
  const navigate = (next: MapPoint) => { setPoint(next); setView({ ...next }); };
  const refreshMap = () => { setRefresh(Date.now()); void refreshPlayers(); };
  const setVisible = (value: boolean) => { visible = value; };
  onSettled(() => { void initialize(); });
  const poll = window.setInterval(() => { if (document.visibilityState === "visible") void refreshPlayers(); }, 5000);
  onCleanup(() => { active = false; ++epoch; lifetime.abort(); worldRequest?.abort(); queryRequest?.abort(); window.clearInterval(poll); });
  return { worlds, loaded, catalog, area, matches, areaRegion, matchRegion, areaSlice, matchedBlock, players, point, view, busy, loading, error, catalogError, playerError, refresh, currentWorld, selectWorld, initialize, loadCatalog, scan, searchBlocks, navigate, setPoint, refreshMap, setVisible };
}

export type MapExplorer = ReturnType<typeof createMapExplorer>;
