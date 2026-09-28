import { createEffect, createMemo, createSignal, For, Show } from "solid-js";
import type { MapExplorer } from "./createMapExplorer";
import { biomeColor, coordinate, displayName } from "./mapMath";
import { predictionCells } from "./predictionMask";

export default function MapPrediction(props: { readonly explorer: MapExplorer }) {
  const [height, setHeight] = createSignal("64");
  createEffect(() => props.explorer.currentWorld(), world => { setHeight(String(Math.max(world?.min_y ?? -64, Math.min(64, world?.max_y ?? 319)))); });
  const valid = () => {
    const y = coordinate(height()), world = props.explorer.currentWorld();
    return world !== undefined && y !== null && y >= world.min_y && y <= world.max_y;
  };
  const cells = createMemo(() => {
    const result = props.explorer.prediction();
    return result ? predictionCells(result, [props.explorer.area(), props.explorer.matches()]) : [];
  });
  const biomes = createMemo(() => [...new Set(cells().map(cell => cell.biome))].sort());
  return <section class="minecraft-tool-panel minecraft-prediction-panel" aria-labelledby="minecraft-prediction-title">
    <h2 id="minecraft-prediction-title">Preview unexplored biomes</h2>
    <p class="minecraft-hint">Predict a fixed Y slice in the selected 128 × 128 block area. Only chunks confirmed ungenerated are shown; this does not predict surface height or player changes.</p>
    <form onSubmit={event => { event.preventDefault(); if (valid()) void props.explorer.predict(Number(height())); }}>
      <label>Prediction Y<input type="number" value={height()} min={props.explorer.currentWorld()?.min_y} max={props.explorer.currentWorld()?.max_y} step="1" required onInput={event => setHeight(event.currentTarget.value)} /></label>
      <button class="minecraft-primary-action" type="submit" disabled={props.explorer.busy() || !valid()}>{props.explorer.predicting() ? "Predicting…" : "Preview selected area"}</button>
    </form>
    <Show when={props.explorer.predictionNotice()}><p class="minecraft-map-warning" role="status">{props.explorer.predictionNotice()}</p></Show>
    <Show when={props.explorer.prediction()}>{result => <div class="minecraft-prediction-receipt">
      <p><strong>Predicted · Y {result().y} · {result().preset === "large_biomes" ? "Large biomes" : "Default biomes"}</strong></p>
      <p class="minecraft-hint">Coverage sampled {new Date(result().sampled_at_ms).toLocaleTimeString()}; expires {new Date(result().expires_at_ms).toLocaleTimeString()}. The server selects the world preset.</p>
      <p class="minecraft-hint">{cells().length} predicted cells. The dark outline marks predicted coverage; no predictions cover generated, excluded, or unknown chunks.</p>
      <p class="minecraft-hint">Biome boundaries may differ from generated terrain. Terrain changes can occur after this snapshot.</p>
      <p class="minecraft-prediction-source">Generator: {result().generator_revision}</p>
      <ul class="minecraft-biome-legend minecraft-prediction-legend"><For each={biomes()}>{biome => <li><span aria-hidden="true" style={{ "background-color": biomeColor(biome) }} />{displayName(biome)} · predicted</li>}</For></ul>
      <button type="button" onClick={() => props.explorer.clearPrediction()}>Clear preview</button>
    </div>}</Show>
  </section>;
}
