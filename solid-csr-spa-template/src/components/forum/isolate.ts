/**
 * Wraps user-authored text in a first-strong isolate (FSI ... PDI) before it is
 * interpolated into a translated sentence, so an English username or a title in
 * the other direction cannot reorder the words or punctuation around it.
 */
export function isolateUserText(value: string): string {
  return `⁨${value}⁩`;
}
