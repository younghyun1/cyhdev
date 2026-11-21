import { For, type Component } from "solid-js";

import type { ArchiveTextSegment } from "../../services/contracts/kcna_archive";

/** Renders text with query matches marked. Always text, never markup. */
const Segments: Component<{ readonly segments: ReadonlyArray<ArchiveTextSegment> }> = (props) => (
  <For each={props.segments}>
    {(segment) => (segment.hit ? <mark class="kcna-mark">{segment.text}</mark> : <>{segment.text}</>)}
  </For>
);

export default Segments;
