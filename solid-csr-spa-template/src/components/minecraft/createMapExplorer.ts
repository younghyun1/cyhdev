import { createSignal, onCleanup, onSettled } from "solid-js";
import type { MinecraftMapData, MinecraftMapQuery, MinecraftPrediction } from "../../generated";
import { ApiContractError } from "../../generated";
import { contractApi } from "../../services/account_api";
import { squaremap, type SquaremapPlayer, type SquaremapSettings, type SquaremapWorld } from "../../services/squaremap";
import { coordinate, scanOrigin, type MapPoint } from "./mapMath";
import { createMapInspection, createMapQueryGate } from "./mapInspectionState";

export type LoadedMap = { readonly world: SquaremapWorld; readonly settings: SquaremapSettings };
export const mapFailure = (cause: unknown) => cause instanceof Error ? cause.message : "The map request failed.";

/** World changes invalidate pending results; hover reads share the manual query gate. */
export function createMapExplorer() {
  const [worlds, setWorlds] = createSignal<SquaremapWorld[]>([]);
  const [loaded, setLoaded] = createSignal<LoadedMap | null>(null);
  const [catalog, setCatalog] = createSignal<MinecraftMapData | null>(null);
  const [area, setArea] = createSignal<MinecraftMapData | null>(null);
  const [matches, setMatches] = createSignal<MinecraftMapData | null>(null);
  const [prediction, setPrediction] = createSignal<MinecraftPrediction | null>(null);
  const [predictionNotice, setPredictionNotice] = createSignal("");
  const [predicting, setPredicting] = createSignal(false);
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
  let active = true, visible = true, epoch = 0, metadataPending = false, playersPending = false;
  const lifetime = new AbortController();
  let worldRequest: AbortController | null = null;
  let queryRequest: AbortController | null = null;
  let predictionRequest: AbortController | null = null, predictionTimer: number | undefined, predictionEpoch = 0;
  const currentWorld = () => catalog()?.worlds.find(world => world.map_id === loaded()?.world.name);
  const available = () => active && visible && document.visibilityState === "visible";
  const gate = createMapQueryGate(available, value => { if (active) setBusy(value); });
  const inspection = createMapInspection({
    allowed: available,
    context: () => ({ world: currentWorld()?.id ?? null, area: area(), areaSlice: areaSlice(), matches: matches(), matchedBlock: matchedBlock(), prediction: prediction() }),
    read: async (world, chunk_x, chunk_z) => (await contractApi.minecraftMapQuery({ body: { kind: "area", world, chunk_x, chunk_z, width: 1, height: 1, y: null } }, { signal: lifetime.signal })).data,
    schedule: job => gate.hover(job), clearQueued: () => gate.clearHover(),
  });
  const clearPrediction = (notice = "") => {
    ++predictionEpoch;
    window.clearTimeout(predictionTimer);
    setPrediction(null); setPredictionNotice(notice);
    queueMicrotask(inspection.refresh);
  };

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
    inspection.invalidate(); gate.invalidate();
    worldRequest?.abort();
    clearPrediction();
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

  const loadCatalog = () => gate.manual({ valid: () => active, run: async () => {
    setCatalogError("");
    try {
      const response = await contractApi.minecraftMapQuery({ body: { kind: "catalog" } }, { signal: lifetime.signal });
      if (active) setCatalog(response.data);
    } catch {
      if (active) { setCatalog(null); setCatalogError("World analysis is unavailable. Terrain and coordinate tools remain available."); }
    } finally { queueMicrotask(inspection.refresh); }
  } });
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

  const query = (body: MinecraftMapQuery) => {
    const revision = epoch;
    gate.manual({ valid: () => revision === epoch, run: async () => {
      const request = new AbortController(); queryRequest = request; setError("");
      try {
        const response = await contractApi.minecraftMapQuery({ body }, { signal: request.signal });
        if (!active || revision !== epoch) return;
        if (body.kind === "area") { setArea(response.data); setAreaRegion(body); setAreaSlice(body.y ?? null); }
        else if (body.kind === "blocks") { setMatches(response.data); setMatchRegion(body); setMatchedBlock(body.block); }
      } catch (cause: unknown) {
        if (active && revision === epoch) setError(mapFailure(cause));
      } finally { queueMicrotask(inspection.refresh); }
    } });
  };

  const scan = (y: number | null) => {
    const world = currentWorld();
    if (world) void query({ kind: "area", world: world.id, ...scanOrigin(point(), 8), y });
  };
  const searchBlocks = (block: string, min_y: number, max_y: number) => {
    const world = currentWorld();
    if (world) void query({ kind: "blocks", world: world.id, ...scanOrigin(point(), 4), block, min_y, max_y });
  };
  const predict = (y: number) => {
    const world = currentWorld();
    if (!world || !available() || !Number.isInteger(y) || y < world.min_y || y > world.max_y) return;
    const worldRevision = epoch;
    const origin = scanOrigin(point(), 8);
    gate.manual({ valid: () => worldRevision === epoch, run: async () => {
      clearPrediction(); setPredicting(true);
      const revision = predictionEpoch, request = new AbortController(); predictionRequest = request;
      try {
        const response = await contractApi.minecraftMapPrediction({ body: { world: world.id, min_x: origin.chunk_x * 16, min_z: origin.chunk_z * 16, y } }, { signal: request.signal });
        if (!active || revision !== predictionEpoch) return;
        const result = response.data;
        const remaining = Math.min(15_000, result.expires_at_ms - Date.now());
        if (result.world !== world.id || result.y !== y || !Number.isFinite(remaining) || remaining <= 0) {
          setPredictionNotice("Preview expired or did not match this world. Request it again."); return;
        }
        setPrediction(result);
        queueMicrotask(inspection.refresh);
        predictionTimer = window.setTimeout(() => clearPrediction("Preview expired. Request a fresh coverage snapshot to show predictions again."), remaining);
      } catch (cause: unknown) {
        if (active && revision === predictionEpoch) setPredictionNotice(cause instanceof ApiContractError && cause.status === 503
          ? "Prediction is unavailable for this world or generator configuration."
          : mapFailure(cause));
      } finally {
        if (active) setPredicting(false);
      }
    } });
  };
  const navigate = (next: MapPoint) => { setPoint(next); setView({ ...next }); inspection.inspect(next); };
  const terrainRefreshing = () => { inspection.invalidate(); if (prediction() || predicting()) clearPrediction("Terrain refreshed. Request a fresh prediction preview."); };
  const refreshMap = () => { terrainRefreshing(); setRefresh(Date.now()); void refreshPlayers(); };
  const setVisible = (value: boolean) => {
    visible = value;
    if (!value) { inspection.invalidate(); gate.invalidate(); clearPrediction(); }
    else if (!catalog() && !catalogError()) loadCatalog();
  };
  const onVisibility = () => {
    if (document.visibilityState !== "visible") { inspection.invalidate(); gate.invalidate(); clearPrediction(); }
    else if (!catalog() && !catalogError()) loadCatalog();
  };
  onSettled(() => { void initialize(); });
  document.addEventListener("visibilitychange", onVisibility);
  const poll = window.setInterval(() => { if (document.visibilityState === "visible") void refreshPlayers(); }, 5000);
  onCleanup(() => { active = false; ++epoch; ++predictionEpoch; inspection.invalidate(); gate.invalidate(); lifetime.abort(); worldRequest?.abort(); queryRequest?.abort(); predictionRequest?.abort(); window.clearTimeout(predictionTimer); window.clearInterval(poll); document.removeEventListener("visibilitychange", onVisibility); });
  return { worlds, loaded, catalog, area, matches, prediction, predictionNotice, predicting, areaRegion, matchRegion, areaSlice, matchedBlock, players, point, view, busy, loading, error, catalogError, playerError, refresh, currentWorld, selectWorld, initialize, loadCatalog, scan, searchBlocks, predict, clearPrediction, terrainRefreshing, navigate, setPoint, refreshMap, setVisible, inspect: inspection.inspect, inspection: inspection.inspection };
}

export type MapExplorer = ReturnType<typeof createMapExplorer>;
