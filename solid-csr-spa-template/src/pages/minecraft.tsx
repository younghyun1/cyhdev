import { createSignal, For, Match, Show, Switch, untrack } from "solid-js";
import type { MinecraftWaypoint } from "../generated";
import MapCanvas from "../components/minecraft/MapCanvas";
import { MapAnalysis, MapBlockSearch, type LayerControls } from "../components/minecraft/MapAnalysis";
import MapTravel from "../components/minecraft/MapTravel";
import MapWaypoints from "../components/minecraft/MapWaypoints";
import MapPrediction from "../components/minecraft/MapPrediction";
import { createMapExplorer, type MapExplorer } from "../components/minecraft/createMapExplorer";
import type { MapPoint } from "../components/minecraft/mapMath";
import "../styles/minecraft.css";

export default function Minecraft() {
  const explorer = createMapExplorer();
  const [original, setOriginal] = createSignal(false);
  return (
    <section class="minecraft-page minecraft-atlas" aria-label="Minecraft world atlas">
      <header class="minecraft-atlas-header">
        <div class="minecraft-atlas-brand"><span class="minecraft-grass-block" aria-hidden="true" /><div><h1>World atlas</h1><p>MINECRAFT EXPLORER</p></div></div>
        <div class="minecraft-atlas-header-actions">
          <Show when={!original()}><label class="minecraft-world-picker">Dimension<select aria-label="Dimension" value={explorer.loaded()?.world.name ?? ""} disabled={explorer.worlds().length === 0} onChange={event => void explorer.selectWorld(event.currentTarget.value)}><For each={explorer.worlds()}>{world => <option value={world.name}>{world.displayName}</option>}</For></select></label></Show>
          <button type="button" aria-pressed={original() ? "true" : "false"} onClick={() => { explorer.setVisible(original()); setOriginal(value => !value); }}>{original() ? "Explorer" : "Original map"}</button>
        </div>
      </header>
      <Show when={!original()} fallback={<iframe class="minecraft-map" src="/minecraft/map/" title="Original Minecraft map" sandbox="allow-scripts allow-popups allow-popups-to-escape-sandbox" allow="clipboard-write *" />}>
        <Show when={explorer.loading()}><div class="minecraft-map-notice" role="status">Loading the world atlas…</div></Show>
        <Show when={explorer.error()}><div class="minecraft-map-notice" role="alert">{explorer.error()} <Show when={!explorer.loaded()}><button type="button" onClick={() => void explorer.initialize()}>Retry map</button></Show></div></Show>
        <Show when={explorer.loaded()} keyed>{_loaded => <ExplorerWorkspace explorer={explorer} />}</Show>
      </Show>
    </section>
  );
}

function ExplorerWorkspace(props: { readonly explorer: MapExplorer }) {
  const explorer = untrack(() => props.explorer);
  const [tab, setTab] = createSignal<"layers" | "blocks" | "places" | "travel">("layers");
  const [biome, setBiome] = createSignal("");
  const [biomes, setBiomes] = createSignal(true), [elevation, setElevation] = createSignal(false);
  const [structures, setStructures] = createSignal(true), [grid, setGrid] = createSignal(false);
  const [measuring, setMeasuring] = createSignal(false), [measureStart, setMeasureStart] = createSignal<MapPoint | null>(null);
  const [waypoints, setWaypoints] = createSignal<readonly MinecraftWaypoint[]>([]);
  const layers: LayerControls = { biome, setBiome, biomes, setBiomes, elevation, setElevation, structures, setStructures, grid, setGrid };
  const tabs = [{ id: "layers", label: "Layers", icon: "▧" }, { id: "blocks", label: "Blocks", icon: "▣" }, { id: "places", label: "Places", icon: "⚑" }, { id: "travel", label: "Travel", icon: "◇" }] as const;
  return <div class="minecraft-atlas-workspace">
    <aside class="minecraft-atlas-sidebar" aria-label="Exploration tools">
      <nav class="minecraft-tool-tabs" aria-label="Map tools"><For each={tabs}>{item => <button type="button" aria-pressed={tab() === item.id ? "true" : "false"} onClick={() => setTab(item.id)}><span aria-hidden="true">{item.icon}</span>{item.label}</button>}</For></nav>
      <div class="minecraft-tool-scroll">
        <Show when={explorer.catalogError()}><p class="minecraft-map-warning">{explorer.catalogError()} <button type="button" disabled={explorer.busy()} onClick={() => void explorer.loadCatalog()}>Retry analysis</button></p></Show>
        <Switch>
          <Match when={tab() === "layers"}><MapPrediction explorer={explorer} /><MapAnalysis explorer={explorer} layers={layers} /></Match>
          <Match when={tab() === "blocks"}><MapBlockSearch explorer={explorer} /></Match>
          <Match when={tab() === "travel"}><MapTravel explorer={explorer} measuring={measuring()} start={measureStart()} onMeasure={enabled => { setMeasuring(enabled); setMeasureStart(enabled ? { ...explorer.point() } : null); }} /></Match>
        </Switch>
        <div hidden={tab() !== "places"}><Show when={explorer.currentWorld()} keyed>{world => <MapWaypoints world={world.id} minY={world.min_y} maxY={world.max_y} point={explorer.point()} onChange={setWaypoints} onNavigate={explorer.navigate} />}</Show><Show when={!explorer.currentWorld()}><p class="minecraft-map-warning">Waypoints need world metadata. Retry world analysis to load them.</p></Show></div>
      </div>
      <div class="minecraft-selection-readout"><span>SELECTED BLOCK</span><strong>X {explorer.point().x} <span>/</span> Z {explorer.point().z}</strong><small>Chunk {Math.floor(explorer.point().x / 16)}, {Math.floor(explorer.point().z / 16)}</small></div>
    </aside>
    <div class="minecraft-atlas-stage">
      <Show when={explorer.loaded()} keyed>{loaded => <MapCanvas mapId={loaded.world.name} settings={loaded.settings} point={explorer.point()} view={explorer.view()} area={explorer.area()} matches={explorer.matches()} prediction={explorer.prediction()} areaRegion={explorer.areaRegion()} matchRegion={explorer.matchRegion()} waypoints={waypoints()} players={explorer.players().filter(player => player.world === loaded.world.name)} biome={biome()} biomeLayer={biomes()} elevation={elevation()} structures={structures()} grid={grid()} measuring={measuring()} measureStart={measureStart()} refresh={explorer.refresh()} onPoint={explorer.setPoint} onTerrainRefresh={explorer.terrainRefreshing} />}</Show>
      <div class="minecraft-map-caption"><span>Click terrain to select an area</span><button type="button" onClick={explorer.refreshMap}>Refresh terrain</button></div>
      <div class="minecraft-map-key"><span class="minecraft-key-waypoint">● Waypoint</span><span class="minecraft-key-player">● Player</span><span class="minecraft-key-match">● Block match</span><Show when={explorer.prediction()}><span class="minecraft-key-prediction">▧ Predicted · Y {explorer.prediction()?.y}</span></Show></div>
      <Show when={explorer.playerError()}><p class="minecraft-player-warning">{explorer.playerError()}</p></Show>
    </div>
  </div>;
}
