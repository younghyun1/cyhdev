import { createEffect, createMemo, createSignal, Show } from "solid-js";
import type { MapExplorer } from "./createMapExplorer";
import { coordinate } from "./mapMath";
import { predictionCells } from "./predictionMask";

export default function MapPrediction(props: { readonly explorer: MapExplorer }) {
  const [height, setHeight] = createSignal("64");
  createEffect(() => props.explorer.predictionY(), value => { setHeight(String(value)); });
  const cells = createMemo(() => {
    const result = props.explorer.prediction();
    return result ? predictionCells(result, [props.explorer.area(), props.explorer.matches()]) : [];
  });
  return <div class="minecraft-prediction-controls">
    <label class="minecraft-toggle"><input type="checkbox" checked={props.explorer.predictionsEnabled()} onChange={event => props.explorer.setPredictionsEnabled(event.currentTarget.checked)} /> Predicted biomes</label>
    <Show when={props.explorer.predictionsEnabled()}>
      <label>Prediction Y<input type="number" value={height()} min={props.explorer.currentWorld()?.min_y ?? -64} max={props.explorer.currentWorld()?.max_y ?? 319} step="1" onInput={event => {
        setHeight(event.currentTarget.value);
        const y = coordinate(event.currentTarget.value), world = props.explorer.currentWorld();
        if (y !== null && world && y >= world.min_y && y <= world.max_y) props.explorer.setPredictionY(y);
      }} /></label>
      <Show when={props.explorer.predicting()}><small role="status">Loading predictions…</small></Show>
      <Show when={props.explorer.predictionNotice()}><p class="minecraft-map-warning" role="status">{props.explorer.predictionNotice()}</p></Show>
      <Show when={props.explorer.prediction()}>{result => <small class="minecraft-prediction-receipt">Predicted · Y {result().y} · {result().preset === "large_biomes" ? "Large biomes" : "Default biomes"} · {cells().length} cells · approximate</small>}</Show>
    </Show>
  </div>;
}
