import type { JSX } from "@solidjs/web";

import type { UiTextKey } from "../../../i18n/keys";
import { tx } from "../../../state/i18n";

type Props = {
  readonly textKey: UiTextKey;
  /** Placeholder values; names listed in `ltr` or `auto` render in their own bidi isolate. */
  readonly params: Record<string, string | number>;
  /** Identifiers such as UUIDs; always left to right. */
  readonly ltr?: readonly string[];
  /** Formatted dates, which follow the direction of the locale that formatted them. */
  readonly auto?: readonly string[];
};

const MARK = "\u0000";

/**
 * A translated sentence with identifiers and dates set off as isolates.
 * Inside a right-to-left sentence, hyphens and digits are neutral and the
 * bidi algorithm reverses a UUID segment by segment, or splits it across
 * lines in the wrong order; an inline-block isolate keeps an identifier one unit. Dates stay inline.
 */
export function IsolatedMessage(props: Props): JSX.Element {
  const parts = () => {
    const isolated = new Set([...(props.ltr ?? []), ...(props.auto ?? [])]);
    const marked: Record<string, string | number> = {};
    for (const [name, value] of Object.entries(props.params)) {
      marked[name] = isolated.has(name) ? `${MARK}${name}${MARK}` : value;
    }
    return tx(props.textKey, marked).split(MARK);
  };
  return (
    <>
      {() =>
        parts().map((part, index) =>
          index % 2 === 0
            ? part
            : (
              <bdi
                class={props.ltr?.includes(part) ? "operations-isolate" : undefined}
                dir={props.ltr?.includes(part) ? "ltr" : "auto"}
              >
                {props.params[part]}
              </bdi>
            )
        )}
    </>
  );
}
