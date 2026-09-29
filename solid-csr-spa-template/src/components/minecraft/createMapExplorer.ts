import { createEffect, createSignal, onCleanup, onSettled, untrack } from "solid-js";
import type { MinecraftMapData, MinecraftMapQuery, MinecraftPrediction } from "../../generated";
import { ApiContractError } from "../../generated";
import { contractApi } from "../../services/account_api";
import { squaremap, type SquaremapPlayer, type SquaremapSettings, type SquaremapWorld } from "../../services/squaremap";
import { coordinate, scanOrigin, type MapPoint } from "./mapMath";
import { createMapInspection, createMapQueryGate } from "./mapInspectionState";

export type LoadedMap = { readonly world: SquaremapWorld; readonly settings: SquaremapSettings };
export const mapFailure = (cause: unknown) => cause instanceof Error ? cause.message : "The map request failed.";
const PREDICTION_RETRY_DELAYS = [2000, 5000, 10_000] as const;

function retryablePredictionFailure(cause: unknown): boolean {
  if (!(cause instanceof ApiContractError)) return cause instanceof Error && cause.name !== "AbortError";
  if (![408, 429, 500, 502, 503, 504].includes(cause.status)) return false;
  try {
    const body: unknown = JSON.parse(cause.body);
    // Missing integration cannot recover through another request; other 503s are ambiguous.
    if (typeof body === "object" && body !== null && "error_code" in body && body.error_code === 86) return false;
  } catch { /* A proxy may return non-JSON transient errors. */ }
  return true;
}

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
  const [predictionsEnabled, setPredictionsEnabled] = createSignal(true);
  const [predictionY, setPredictionY] = createSignal(64);
  const [areaRegion, setAreaRegion] = createSignal<ReturnType<typeof scanOrigin> | null>(null);
  const [matchRegion, setMatchRegion] = createSignal<ReturnType<typeof scanOrigin> | null>(null);
  const [areaSlice, setAreaSlice] = createSignal<number | null>(null);
  const [matchedBlock, setMatchedBlock] = createSignal("");
  const [players, setPlayers] = createSignal<SquaremapPlayer[]>([]);
  const [point, setPoint] = createSignal<MapPoint>({ x: 0, z: 0 });
  const [selectedRegion, setSelectedRegion] = createSignal<ReturnType<typeof scanOrigin> | null>(null);
  const [view, setView] = createSignal<MapPoint>({ x: 0, z: 0 });
  const [busy, setBusy] = createSignal(false);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal("");
  const [catalogError, setCatalogError] = createSignal("");
  const [playerError, setPlayerError] = createSignal("");
  const [refresh, setRefresh] = createSignal(0);
  let active = true, epoch = 0, metadataPending = false, playersPending = false;
  const lifetime = new AbortController();
  let worldRequest: AbortController | null = null;
  let queryRequest: AbortController | null = null;
  let predictionRequest: AbortController | null = null, predictionTimer: number | undefined, predictionEpoch = 0;
  let predictionRetryTimer: number | undefined;
  let automaticTimer: number | undefined, automaticCenter: MapPoint | null = null;
  let automaticKey = "";
  const currentWorld = () => catalog()?.worlds.find(world => world.map_id === loaded()?.world.name);
  const available = () => active && document.visibilityState === "visible";
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
    window.clearTimeout(predictionRetryTimer);
    setPrediction(null); setPredictionNotice(notice); setPredicting(false);
    queueMicrotask(inspection.refresh);
  };

  const refreshPlayers = async () => {
    if (playersPending || !available()) return;
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
    automaticCenter = null; automaticKey = ""; window.clearTimeout(automaticTimer);
    worldRequest?.abort();
    clearPrediction();
    const request = new AbortController(); worldRequest = request;
    setLoaded(null); setArea(null); setMatches(null); setAreaRegion(null); setMatchRegion(null); setSelectedRegion(null); setLoading(true); setError("");
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
    if (world) void query({ kind: "area", world: world.id, ...(selectedRegion() ?? scanOrigin(point(), 8)), y });
  };
  const searchBlocks = (block: string, min_y: number, max_y: number) => {
    const world = currentWorld();
    const region = selectedRegion() ?? scanOrigin(point(), 4);
    if (region.width > 4 || region.height > 4) { setError("Select an area no larger than 64 × 64 blocks for block search."); return; }
    if (world) void query({ kind: "blocks", world: world.id, ...region, block, min_y, max_y });
  };
  const requestPrediction = (y: number, center: MapPoint, automatic: boolean, attempt = 0) => {
    const staticWorld = currentWorld();
    if (!staticWorld || !available() || !Number.isInteger(y) || y < staticWorld.min_y || y > staticWorld.max_y) return;
    const worldRevision = epoch;
    const origin = scanOrigin(center, 8), requestedKey = `${staticWorld.id}:${origin.chunk_x}:${origin.chunk_z}:${y}`;
    if (automatic && requestedKey === automaticKey) return;
    const job = { valid: () => worldRevision === epoch && (!automatic || predictionsEnabled()), run: async () => {
      automaticKey = requestedKey;
      clearPrediction(); setPredicting(true);
      const revision = predictionEpoch, request = new AbortController(); predictionRequest = request;
      const retry = () => {
        const delay = PREDICTION_RETRY_DELAYS[attempt];
        if (!automatic || delay === undefined) return;
        predictionRetryTimer = window.setTimeout(() => {
          if (!available() || !predictionsEnabled() || revision !== predictionEpoch) return;
          automaticKey = "";
          requestPrediction(y, center, true, attempt + 1);
        }, delay);
      };
      try {
        const response = await contractApi.minecraftMapPrediction({ body: { world: staticWorld.id, min_x: origin.chunk_x * 16, min_z: origin.chunk_z * 16, y } }, { signal: request.signal });
        if (!active || revision !== predictionEpoch) return;
        const result = response.data;
        const remaining = Math.min(15_000, result.expires_at_ms - Date.now());
        if (result.world !== staticWorld.id || result.y !== y || !Number.isFinite(remaining) || remaining <= 0) {
          setPredictionNotice("Preview expired or did not match this world. Request it again.");
          if (result.world === staticWorld.id && result.y === y && Number.isFinite(remaining) && remaining <= 0) retry();
          return;
        }
        setPrediction(result);
        queueMicrotask(inspection.refresh);
        predictionTimer = window.setTimeout(() => { clearPrediction(); automaticKey = ""; schedulePrediction(); }, remaining);
      } catch (cause: unknown) {
        if (active && revision === predictionEpoch) {
          setPredictionNotice(cause instanceof ApiContractError && cause.status === 503
            ? "Prediction is unavailable for this world or generator configuration."
            : mapFailure(cause));
          if (retryablePredictionFailure(cause)) retry();
        }
      } finally {
        if (active) setPredicting(false);
      }
    } };
    if (automatic) gate.background(job); else gate.manual(job);
  };
  const schedulePrediction = () => {
    window.clearTimeout(automaticTimer); gate.clearBackground();
    if (!predictionsEnabled() || !available() || !currentWorld() || loaded()?.world.type !== "normal") return;
    automaticTimer = window.setTimeout(() => requestPrediction(predictionY(), automaticCenter ?? point(), true), 700);
  };
  const mapViewChanged = (center: MapPoint) => {
    const previous = automaticCenter; automaticCenter = center;
    if (!previous || Math.floor(previous.x / 16) !== Math.floor(center.x / 16) || Math.floor(previous.z / 16) !== Math.floor(center.z / 16)) {
      clearPrediction(); automaticKey = ""; schedulePrediction();
    }
  };
  const predict = (y: number) => requestPrediction(y, point(), false);
  createEffect(() => [loaded(), catalog(), predictionsEnabled(), predictionY()] as const, () => untrack(() => {
    clearPrediction(); automaticKey = ""; schedulePrediction();
  }));
  const selectPoint = (next: MapPoint) => { setSelectedRegion(null); setPoint(next); };
  const selectRegion = (region: ReturnType<typeof scanOrigin>) => {
    setSelectedRegion(region); setPoint({ x: region.chunk_x * 16 + region.width * 8, z: region.chunk_z * 16 + region.height * 8 });
  };
  const navigate = (next: MapPoint) => { selectPoint(next); setView({ ...next }); inspection.inspect(next); };
  const terrainRefreshing = () => { inspection.invalidate(); clearPrediction(); automaticKey = ""; schedulePrediction(); };
  const refreshMap = () => { terrainRefreshing(); setRefresh(value => value + 1); void refreshPlayers(); };
  const onVisibility = () => {
    if (document.visibilityState !== "visible") { inspection.invalidate(); gate.invalidate(); clearPrediction(); window.clearTimeout(automaticTimer); automaticKey = ""; }
    else { if (!catalog() && !catalogError()) loadCatalog(); schedulePrediction(); }
  };
  onSettled(() => { void initialize(); });
  document.addEventListener("visibilitychange", onVisibility);
  const poll = window.setInterval(() => { if (document.visibilityState === "visible") void refreshPlayers(); }, 5000);
  onCleanup(() => { active = false; ++epoch; ++predictionEpoch; inspection.invalidate(); gate.invalidate(); lifetime.abort(); worldRequest?.abort(); queryRequest?.abort(); predictionRequest?.abort(); window.clearTimeout(predictionTimer); window.clearTimeout(predictionRetryTimer); window.clearTimeout(automaticTimer); window.clearInterval(poll); document.removeEventListener("visibilitychange", onVisibility); });
  return { worlds, loaded, catalog, area, matches, prediction, predictionNotice, predicting, predictionsEnabled, setPredictionsEnabled, predictionY, setPredictionY, mapViewChanged, areaRegion, matchRegion, areaSlice, matchedBlock, players, point, selectedRegion, selectRegion, view, busy, loading, error, catalogError, playerError, refresh, currentWorld, selectWorld, initialize, loadCatalog, scan, searchBlocks, predict, clearPrediction, terrainRefreshing, navigate, setPoint: selectPoint, refreshMap, inspect: inspection.inspect, inspection: inspection.inspection };
}

export type MapExplorer = ReturnType<typeof createMapExplorer>;
