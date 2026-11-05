const URL_PATTERN = /(?:https?:\/\/|www\.)\S+/giu;
const RTL_LETTER = /[\p{Script=Arabic}\p{Script=Hebrew}\p{Script=Syriac}\p{Script=Thaana}\p{Script=Nko}]/u;
const ANY_LETTER = /\p{L}/u;

/**
 * Base direction for a chat message of unknown language.
 *
 * `dir="auto"` follows the first strong character, so an Arabic sentence that
 * opens with a Latin word ("Rust أم Go؟") would be laid out left to right and
 * its closing punctuation would land on the wrong side. This counts words
 * instead and lets the majority script decide; URLs are ignored because their
 * Latin letters say nothing about the language of the sentence. A tie or a
 * message without words stays `auto`.
 */
export function messageDirection(body: string): "rtl" | "ltr" | "auto" {
  let rtl = 0;
  let other = 0;
  for (const word of body.replace(URL_PATTERN, " ").split(/[^\p{L}\p{M}]+/u)) {
    const first = Array.from(word).find((char) => ANY_LETTER.test(char));
    if (first === undefined) continue;
    if (RTL_LETTER.test(first)) rtl += 1;
    else other += 1;
  }
  if (rtl > other) return "rtl";
  if (other > rtl) return "ltr";
  return "auto";
}
