import { Show, createMemo, createSignal } from "solid-js";
import type { JSX } from "@solidjs/web";
import { customColor } from "../state/accent";
import { setCustomAccent } from "../state/custom_accent";
import { t } from "../state/i18n";
import { theme } from "../state/theme";
import { deriveCustomAccent, formatRgb, parseRgbInput } from "../utils/customAccent";

/** Color values quoted inside a translated note: `255, 120, 40` and `#ff7828`. */
const COLOR_EXAMPLE = /(#[0-9a-fA-F]{3,8}|\d{1,3}(?:,\s*\d{1,3}){2})/;

/**
 * Wraps each quoted color value in a left-to-right island. In a right-to-left
 * sentence the bidi rules would otherwise put the `#` after its digits and
 * reverse the order of the three numbers, so the example would not match what
 * the field accepts.
 */
function isolateColorExamples(text: string): Array<string | JSX.Element> {
  return text
    .split(COLOR_EXAMPLE)
    .map((part, index) => (index % 2 === 1 ? <bdi dir="ltr">{part}</bdi> : part));
}

/**
 * The visitor's own color: a native picker and a typed RGB value. A value is
 * applied as soon as it parses; a complaint waits until the field is left, so
 * typing `255, 1` on the way to `255, 120, 40` is not an error.
 */
export default function CustomAccentFields(): JSX.Element {
  const [draft, setDraft] = createSignal(formatRgb(customColor() ?? "#000000"));
  const [invalid, setInvalid] = createSignal(false);
  const adjusted = createMemo(() => {
    const color = customColor();
    return color !== null && deriveCustomAccent(color)[theme()].accent !== color;
  });

  return (
    <>
      <div class="accent-custom">
        <input
          class="accent-custom-color"
          type="color"
          value={customColor() ?? "#000000"}
          aria-label={t("accent.custom")}
          onInput={(event) => {
            setCustomAccent(event.currentTarget.value);
            setDraft(formatRgb(event.currentTarget.value));
            setInvalid(false);
          }}
        />
        <input
          class="accent-custom-text"
          dir="ltr"
          type="text"
          value={draft()}
          maxlength={24}
          autocomplete="off"
          spellcheck="false"
          placeholder="255, 120, 40"
          aria-label={t("accent.custom_rgb")}
          aria-invalid={invalid() ? "true" : undefined}
          onInput={(event) => {
            setDraft(event.currentTarget.value);
            const color = parseRgbInput(event.currentTarget.value);
            if (!color) return;
            setCustomAccent(color);
            setInvalid(false);
          }}
          onChange={(event) => setInvalid(parseRgbInput(event.currentTarget.value) === null)}
        />
      </div>
      <Show when={invalid()}>
        <p class="accent-custom-note accent-custom-note--error" role="alert">
          {isolateColorExamples(t("accent.custom_invalid"))}
        </p>
      </Show>
      <Show when={!invalid() && adjusted()}>
        <p class="accent-custom-note">{isolateColorExamples(t("accent.custom_adjusted"))}</p>
      </Show>
    </>
  );
}
