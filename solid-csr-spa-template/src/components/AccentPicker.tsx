import { For } from "solid-js";
import type { JSX } from "@solidjs/web";
import type { UiTextKey } from "../i18n/keys";
import { ACCENTS, type Accent, accent, setAccent } from "../state/accent";
import { t } from "../state/i18n";

const LABELS: Readonly<Record<Accent, UiTextKey>> = {
  amber: "accent.amber",
  lagoon: "accent.lagoon",
  cobalt: "accent.cobalt",
  orchid: "accent.orchid",
  graphite: "accent.graphite",
};

/**
 * Accent swatches as a native radio group, so arrow keys move between
 * presets. Each swatch previews its color through `data-swatch` styles in
 * header.css rather than reading the active accent token.
 */
export default function AccentPicker(props: { name: string }): JSX.Element {
  return (
    <div class="accent-picker" role="radiogroup" aria-label={t("accent.label")}>
      <For each={ACCENTS}>
        {(name) => (
          <label class="accent-swatch" data-swatch={name} title={t(LABELS[name])}>
            <input
              type="radio"
              name={props.name}
              value={name}
              checked={accent() === name}
              onChange={() => setAccent(name)}
            />
            <span class="sr-only">{t(LABELS[name])}</span>
          </label>
        )}
      </For>
    </div>
  );
}
