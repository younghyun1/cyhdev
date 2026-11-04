import { Errored, For, Loading, Show, lazy } from "solid-js";
import type { JSX } from "@solidjs/web";
import { ACCENTS, accent, setAccent } from "../state/accent";
import { t } from "../state/i18n";

/** Few visitors set their own color, so its code stays out of the initial chunk. */
const CustomAccentFields = lazy(() => import("./CustomAccentFields"));

function selectCustom(radio: HTMLInputElement): void {
  void import("../state/custom_accent")
    .then(({ selectCustomAccent }) => selectCustomAccent())
    .catch((err: unknown) => {
      console.error("Failed to load the custom accent:", err);
      radio.checked = false;
    });
}

/**
 * Accent swatches as a native radio group, so arrow keys move between
 * presets. Each swatch previews its color through `data-swatch` styles in
 * header.css rather than reading the active accent token. The last choice is
 * the visitor's own color, whose fields appear once it is selected.
 */
export default function AccentPicker(props: { name: string }): JSX.Element {
  return (
    <div class="accent-controls">
      <div class="accent-picker" role="radiogroup" aria-label={t("accent.label")}>
        <For each={ACCENTS}>
          {(name) => (
            <label class="accent-swatch" data-swatch={name} title={t(`accent.${name}`)}>
              <input
                type="radio"
                name={props.name}
                value={name}
                checked={accent() === name}
                onChange={() => setAccent(name)}
              />
              <span class="sr-only">{t(`accent.${name}`)}</span>
            </label>
          )}
        </For>
        <label class="accent-swatch" data-swatch="custom" title={t("accent.custom")}>
          <input
            type="radio"
            name={props.name}
            value="custom"
            checked={accent() === "custom"}
            onChange={(event) => selectCustom(event.currentTarget)}
          />
          <span class="sr-only">{t("accent.custom")}</span>
        </label>
      </div>
      <Show when={accent() === "custom"}>
        <Errored fallback={null}>
          <Loading fallback={null}>
            <CustomAccentFields />
          </Loading>
        </Errored>
      </Show>
    </div>
  );
}
