import { createEffect, createSignal, onCleanup, onSettled, untrack } from "solid-js";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
import type { MinecraftMapData, MinecraftPrediction, MinecraftWaypoint } from "../../generated";
import type { SquaremapPlayer, SquaremapSettings } from "../../services/squaremap";
import { biomeColor, displayName, elevationColor, scanOrigin, WORLD_LIMIT, type MapPoint } from "./mapMath";
import { predictionBoundary, predictionCells, predictionEdgeCells } from "./predictionMask";

type Props = {
  readonly mapId: string;
  readonly settings: SquaremapSettings;
  readonly point: MapPoint;
  readonly view: MapPoint;
  readonly area: MinecraftMapData | null;
  readonly matches: MinecraftMapData | null;
  readonly prediction: MinecraftPrediction | null;
  readonly areaRegion: ReturnType<typeof scanOrigin> | null;
  readonly matchRegion: ReturnType<typeof scanOrigin> | null;
  readonly waypoints: readonly MinecraftWaypoint[];
  readonly players: readonly SquaremapPlayer[];
  readonly biome: string;
  readonly biomeLayer: boolean;
  readonly elevation: boolean;
  readonly structures: boolean;
  readonly grid: boolean;
  readonly measuring: boolean;
  readonly measureStart: MapPoint | null;
  readonly refresh: number;
  readonly onPoint: (point: MapPoint) => void;
  readonly onInspect: (point: MapPoint | null) => void;
  readonly onTerrainRefresh: () => void;
};

/** All plugin text enters Leaflet through DOM text nodes, never HTML strings. */
function label(value: string): HTMLElement {
  const node = document.createElement("span");
  node.textContent = value;
  return node;
}

export default function MapCanvas(props: Props) {
  let element: HTMLDivElement | undefined;
  let map: L.Map | undefined;
  let tiles: L.TileLayer | undefined;
  let observer: ResizeObserver | undefined;
  let refreshTimer: number | undefined;
  let predictionRenderer: L.SVG | undefined;
  const [ready, setReady] = createSignal(false);
  const predicted = L.layerGroup(), sampled = L.layerGroup(), markers = L.layerGroup(), selection = L.layerGroup(), grid = L.layerGroup();
  // The parent keys this component by its loaded world, so projection settings are immutable here.
  const settings = untrack(() => props.settings), worldMapId = untrack(() => props.mapId);
  const scale = 2 ** settings.maxZoom;
  const position = (point: MapPoint): L.LatLngTuple => [-point.z / scale, point.x / scale];
  const bounds = (x: number, z: number, endX: number, endZ: number): L.LatLngBoundsExpression => [position({ x, z }), position({ x: endX, z: endZ })];
  const invalidate = () => map?.invalidateSize({ animate: false });
  const clearInspection = () => untrack(() => props.onInspect(null));
  const eventPoint = (event: MouseEvent): MapPoint | null => {
    if (!map) return null;
    const cursor = map.mouseEventToLatLng(event);
    const x = Math.floor(cursor.lng * scale), z = Math.floor(-cursor.lat * scale);
    return Math.abs(x) <= WORLD_LIMIT && Math.abs(z) <= WORLD_LIMIT ? { x, z } : null;
  };
  // Leaflet canvas events throttle movement and snap small marker hits to their centers.
  const inspectCursor = (event: MouseEvent) => untrack(() => {
    if (event.buttons !== 0 || event.target instanceof Element && event.target.closest(".leaflet-control")) { clearInspection(); return; }
    props.onInspect(eventPoint(event));
  });

  const drawGrid = () => {
    grid.clearLayers();
    if (!map || !untrack(() => props.grid)) return;
    const visible = map.getBounds();
    const minX = Math.max(-WORLD_LIMIT, Math.floor(visible.getWest() * scale / 16) * 16);
    const maxX = Math.min(WORLD_LIMIT, Math.ceil(visible.getEast() * scale / 16) * 16);
    const minZ = Math.max(-WORLD_LIMIT, Math.floor(-visible.getNorth() * scale / 16) * 16);
    const maxZ = Math.min(WORLD_LIMIT, Math.ceil(-visible.getSouth() * scale / 16) * 16);
    if ((maxX - minX) / 16 + (maxZ - minZ) / 16 > 128) return;
    const style = { color: "#fff0ba", opacity: 0.4, weight: 1, interactive: false };
    for (let x = minX; x <= maxX; x += 16) L.polyline([position({ x, z: minZ }), position({ x, z: maxZ })], style).addTo(grid);
    for (let z = minZ; z <= maxZ; z += 16) L.polyline([position({ x: minX, z }), position({ x: maxX, z })], style).addTo(grid);
  };

  onSettled(() => {
    if (!element) return;
    map = L.map(element, { crs: L.CRS.Simple, attributionControl: false, preferCanvas: true, minZoom: 0, maxZoom: settings.maxZoom + settings.extraZoom, zoomControl: true });
    map.setView(position(untrack(() => props.view)), settings.defaultZoom);
    tiles = L.tileLayer(`/minecraft/map/tiles/${worldMapId}/{z}/{x}_{y}.png`, { tileSize: 512, minNativeZoom: 0, maxNativeZoom: settings.maxZoom, noWrap: true, keepBuffer: 1, errorTileUrl: "data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=" }).addTo(map);
    map.createPane("minecraft-predictions");
    map.createPane("minecraft-selection-footprint");
    predictionRenderer = L.svg({ pane: "minecraft-predictions" });
    predicted.addTo(map); sampled.addTo(map); grid.addTo(map); markers.addTo(map); selection.addTo(map);
    map.on("click", (event: L.LeafletMouseEvent) => untrack(() => {
      const point = eventPoint(event.originalEvent);
      if (point) { props.onPoint(point); props.onInspect(point); }
    }));
    // Leaving or moving the map cancels a stationary inspection; panning itself never scans.
    element.addEventListener("mousemove", inspectCursor);
    element.addEventListener("mouseleave", clearInspection);
    map.on("dragstart zoomstart", clearInspection);
    map.on("moveend", drawGrid);
    observer = typeof ResizeObserver === "undefined" ? undefined : new ResizeObserver(invalidate);
    observer?.observe(element);
    refreshTimer = window.setInterval(() => {
      if (document.visibilityState === "visible" && tiles && !tiles.isLoading()) {
        untrack(() => props.onTerrainRefresh());
        tiles.setUrl(`/minecraft/map/tiles/${worldMapId}/{z}/{x}_{y}.png?refresh=${Date.now()}`);
      }
    }, 30_000);
    setReady(true);
  });
  onCleanup(() => { window.clearInterval(refreshTimer); element?.removeEventListener("mousemove", inspectCursor); element?.removeEventListener("mouseleave", clearInspection); observer?.disconnect(); map?.remove(); map = undefined; });

  createEffect(() => [ready(), props.view] as const, ([mounted, point]) => {
    if (mounted) map?.panTo(position(point), { animate: false });
  });
  createEffect(() => [ready(), props.grid] as const, () => untrack(drawGrid));
  createEffect(() => [ready(), props.refresh] as const, ([mounted, refresh]) => {
    if (mounted && refresh > 0 && tiles && !tiles.isLoading()) tiles.setUrl(`/minecraft/map/tiles/${worldMapId}/{z}/{x}_{y}.png?refresh=${refresh}`);
  });
  createEffect(() => [ready(), props.prediction, props.area, props.matches] as const, ([mounted, prediction, area, matches]) => {
    if (!mounted || !predictionRenderer) return;
    predicted.clearLayers();
    if (!prediction) return;
    const cells = predictionCells(prediction, [area, matches]);
    const edgeCells = predictionEdgeCells(cells);
    for (const cell of cells) {
      L.rectangle(bounds(cell.x, cell.z, cell.x + prediction.step, cell.z + prediction.step), {
        renderer: predictionRenderer, pane: "minecraft-predictions", className: `minecraft-prediction-cell${edgeCells.has(`${cell.x},${cell.z}`) ? " minecraft-prediction-edge-cell" : ""}`, stroke: false,
        fillColor: biomeColor(cell.biome),
      }).bindTooltip(label(`Predicted ${displayName(cell.biome)} · fixed Y ${prediction.y} · ${cell.x}, ${cell.z}`)).addTo(predicted);
    }
    const perimeter = predictionBoundary(cells).map(edge => edge.map(position));
    if (perimeter.length > 0) for (const className of ["minecraft-prediction-boundary-casing", "minecraft-prediction-boundary"]) {
      L.polyline(perimeter, { renderer: predictionRenderer, pane: "minecraft-predictions", className, interactive: false }).addTo(predicted);
    }
  });
  createEffect(() => [ready(), props.area, props.biome, props.biomeLayer, props.elevation] as const, ([mounted, area, biome, showBiomes, elevation]) => {
    if (!mounted) return;
    sampled.clearLayers();
    if (!area || (!showBiomes && !elevation)) return;
    const heights = area.cells.map(cell => cell.y);
    const min = Math.min(...heights), max = Math.max(...heights);
    for (const cell of area.cells) {
      const selected = !biome || cell.biome === biome;
      const color = elevation ? elevationColor(cell.y, min, max) : biomeColor(cell.biome);
      L.rectangle(bounds(cell.x, cell.z, cell.x + 4, cell.z + 4), { stroke: false, fillColor: selected ? color : "#111111", fillOpacity: selected ? 0.65 : 0.75 })
        .bindTooltip(label(`${displayName(cell.biome)} · surface Y ${cell.y} · ${cell.x}, ${cell.z}`)).addTo(sampled);
    }
  });
  createEffect(() => [ready(), props.area, props.matches, props.waypoints, props.players, props.structures] as const, ([mounted, area, matches, waypoints, players, structures]) => {
    if (!mounted) return;
    markers.clearLayers();
    if (structures) for (const structure of area?.structures ?? []) {
      L.rectangle(bounds(structure.min_x, structure.min_z, structure.max_x + 1, structure.max_z + 1), { color: "#ffe19c", weight: 2, fillOpacity: 0.1 })
        .bindTooltip(label(`${displayName(structure.kind)} · Y ${structure.min_y} to ${structure.max_y}`)).addTo(markers);
    }
    for (const match of matches?.matches ?? []) L.circleMarker(position(match), { radius: 4, color: "#111111", weight: 1, fillColor: "#68e4ef", fillOpacity: 1 })
      .bindTooltip(label(`Block · ${match.x}, ${match.y}, ${match.z}`)).addTo(markers);
    for (const waypoint of waypoints) L.circleMarker(position(waypoint), { radius: 7, color: "#20130d", weight: 2, fillColor: "#ffcd6e", fillOpacity: 1 })
      .bindTooltip(label(`${waypoint.name} · ${waypoint.x}, ${waypoint.y}, ${waypoint.z}${waypoint.description ? ` · ${waypoint.description}` : ""}`)).addTo(markers);
    for (const player of players) L.circleMarker(position(player), { radius: 5, color: "#172214", weight: 2, fillColor: "#a2ee8d", fillOpacity: 1 })
      .bindTooltip(label(player.name)).addTo(markers);
  });
  createEffect(() => [ready(), props.point, props.measuring, props.measureStart, props.areaRegion, props.matchRegion] as const, ([mounted, point, measuring, start, areaRegion, matchRegion]) => {
    if (!mounted) return;
    selection.clearLayers();
    const origin = scanOrigin(point, 8);
    L.rectangle(bounds(origin.chunk_x * 16, origin.chunk_z * 16, (origin.chunk_x + 8) * 16, (origin.chunk_z + 8) * 16), { pane: "minecraft-selection-footprint", color: "#ffffff", weight: 1, dashArray: "5 5", fill: false, interactive: false }).addTo(selection);
    L.circleMarker(position(point), { radius: 6, color: "#ffffff", weight: 2, fill: false, interactive: false }).addTo(selection);
    for (const [region, color, title] of [[areaRegion, "#ffcd6e", "Surveyed area"], [matchRegion, "#68e4ef", "Block search area"]] as const) {
      if (region) L.rectangle(bounds(region.chunk_x * 16, region.chunk_z * 16, (region.chunk_x + region.width) * 16, (region.chunk_z + region.height) * 16), { color, weight: 2, fill: false })
        .bindTooltip(label(title)).addTo(selection);
    }
    if (measuring && start) L.polyline([position(start), position(point)], { color: "#ffffff", dashArray: "5 5", weight: 2, interactive: false }).addTo(selection);
  });

  return <div class="minecraft-atlas-canvas" ref={node => { element = node; }} role="region" aria-label="Interactive Minecraft terrain map" />;
}
