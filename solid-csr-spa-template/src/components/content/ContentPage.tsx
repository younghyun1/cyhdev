import { For, Show, createMemo, createSignal, onSettled } from "solid-js";
import type { JSX } from "@solidjs/web";
import type { UiLocale } from "../../i18n/keys";
import { localeDirection } from "../../i18n/locales";
import type {
  ContentBlock,
  ContentPage as ContentPageData,
  SectionContent,
} from "../../pages/about-locales/content_types";
import { estimateReadingMinutes } from "../../utils/readingTime";
import { inlineToPlainText } from "../../utils/inlineMarkup";
import ContentBlocks from "./ContentBlocks";
import InlineText from "./InlineText";
import "../../styles/content-page.css";

function blockText(block: ContentBlock): string[] {
  switch (block.kind) {
    case "paragraph":
      return [block.text];
    case "list":
      return [...block.items];
    case "callout":
      return [block.title ?? "", block.text];
    case "stats":
      return block.items.flatMap((item) => [item.value, item.label]);
    case "roles":
      return block.roles.flatMap((role) => [
        role.organization,
        role.title,
        role.summary ?? "",
        ...role.highlights,
        ...(role.projects ?? []).flatMap((project) => [project.name, ...project.highlights]),
      ]);
    case "cards":
      return block.cards.flatMap((card) => [card.title, card.text]);
    case "pipeline":
      return block.steps.flatMap((step) => [step.label, step.detail]);
    case "tags":
      return block.groups.flatMap((group) => [group.label, ...group.items]);
    case "bars":
      return [block.caption];
    case "table":
      return [...block.columns, ...block.rows.flat()];
  }
}

/** Every readable string on a page, used for the reading-time estimate. */
export function pagePlainText(page: ContentPageData): string {
  return [
    page.title,
    page.lede,
    ...page.sections.flatMap((section) => [
      section.heading,
      ...section.blocks.flatMap(blockText),
    ]),
  ]
    .map(inlineToPlainText)
    .join(" ");
}

type ContentPageProps<Id extends string> = {
  readonly locale: UiLocale;
  readonly content: SectionContent<Id>;
  readonly pageId: Id;
  readonly pageIds: ReadonlyArray<Id>;
  readonly basePath: string;
  /** Extra class on `<main>`, kept for existing page hooks such as `.about-page`. */
  readonly pageClass?: string;
  /** Optional media beside the hero, such as the portrait on About. */
  readonly aside?: JSX.Element;
  /** Content rendered after the sections and before the pager. */
  readonly children?: JSX.Element;
};

export default function ContentPage<Id extends string>(
  props: ContentPageProps<Id>,
): JSX.Element {
  const page = createMemo(() => props.content.pages[props.pageId]);
  const hrefFor = (id: Id) => (id === props.pageIds[0] ? props.basePath : `${props.basePath}/${id}`);
  const labelFor = (id: Id) =>
    id === props.pageIds[0] ? props.content.chrome.overview : props.content.pages[id].title;
  const position = createMemo(() => props.pageIds.indexOf(props.pageId));
  const previous = createMemo(() => props.pageIds[position() - 1]);
  const next = createMemo(() => props.pageIds[position() + 1]);
  const readingMinutes = createMemo(() =>
    estimateReadingMinutes(pagePlainText(page()), props.locale),
  );
  const [activeSection, setActiveSection] = createSignal<string | null>(null);

  // Scroll-spy for the table of contents. The observer watches only this
  // page's sections and is disconnected when the page unmounts.
  onSettled(() => {
    if (typeof IntersectionObserver === "undefined") return;
    const sections = Array.from(
      document.querySelectorAll<HTMLElement>(".cp-body > .cp-section[id]"),
    );
    if (sections.length === 0) return;
    const visible = new Map<string, number>();
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) visible.set(entry.target.id, entry.boundingClientRect.top);
          else visible.delete(entry.target.id);
        }
        let best: string | null = null;
        let bestTop = Number.POSITIVE_INFINITY;
        for (const [id, top] of visible) {
          if (top < bestTop) {
            best = id;
            bestTop = top;
          }
        }
        if (best) setActiveSection(best);
      },
      { rootMargin: "-20% 0px -55% 0px" },
    );
    for (const section of sections) observer.observe(section);
    return () => observer.disconnect();
  });

  return (
    <main lang={props.locale} dir={localeDirection(props.locale)} class={["app-page cp-page", props.pageClass]}>
      <div class="reading-progress" aria-hidden="true" />
      <div class="cp-shell">
        <nav class="cp-tabs" aria-label={props.content.chrome.sectionNavigation}>
          <ul class="cp-tab-list">
            <For each={props.pageIds}>
              {(id) => (
                <li>
                  <a
                    href={hrefFor(id)}
                    class="cp-tab"
                    aria-current={id === props.pageId ? "page" : undefined}
                  >
                    {labelFor(id)}
                  </a>
                </li>
              )}
            </For>
          </ul>
        </nav>

        <header class="cp-hero">
          <div class="cp-hero-text">
            <p class="cp-eyebrow">{page().eyebrow}</p>
            <h1 class="cp-title">{page().title}</h1>
            <p class="cp-lede">
              <InlineText text={page().lede} />
            </p>
            <p class="cp-meta">
              <span>
                {props.content.chrome.updated}: {props.content.updated}
              </span>
              <span aria-hidden="true">·</span>
              <span>
                {props.content.chrome.readingTime.replace(
                  "{minutes}",
                  String(readingMinutes()),
                )}
              </span>
            </p>
          </div>
          <Show when={props.aside}>
            <div class="cp-hero-aside">{props.aside}</div>
          </Show>
        </header>

        <div class="cp-layout">
          <nav class="cp-toc" aria-label={props.content.chrome.onThisPage}>
            <p class="cp-toc-title">{props.content.chrome.onThisPage}</p>
            <ol class="cp-toc-list">
              <For each={page().sections}>
                {(section) => (
                  <li>
                    <a
                      href={`#${section.id}`}
                      class="cp-toc-link"
                      data-active={activeSection() === section.id ? "" : undefined}
                    >
                      {section.heading}
                    </a>
                  </li>
                )}
              </For>
            </ol>
          </nav>

          <div class="cp-body">
            <For each={page().sections}>
              {(section) => (
                <section
                  id={section.id}
                  class="cp-section"
                  aria-labelledby={`${section.id}-heading`}
                >
                  <h2 id={`${section.id}-heading`} class="cp-section-heading">
                    <a href={`#${section.id}`} class="cp-anchor" aria-hidden="true" tabindex="-1">
                      #
                    </a>
                    {section.heading}
                  </h2>
                  <ContentBlocks blocks={section.blocks} />
                </section>
              )}
            </For>

            {props.children}

            <nav
              class="cp-pager"
              aria-label={`${props.content.chrome.previous} / ${props.content.chrome.next}`}
            >
              <Show when={previous()} fallback={<span />}>
                {(id) => (
                  <a href={hrefFor(id())} class="cp-pager-link" rel="prev">
                    <span class="cp-pager-direction">← {props.content.chrome.previous}</span>
                    <span class="cp-pager-title">{labelFor(id())}</span>
                  </a>
                )}
              </Show>
              <Show when={next()}>
                {(id) => (
                  <a href={hrefFor(id())} class="cp-pager-link cp-pager-next" rel="next">
                    <span class="cp-pager-direction">{props.content.chrome.next} →</span>
                    <span class="cp-pager-title">{labelFor(id())}</span>
                  </a>
                )}
              </Show>
            </nav>
          </div>
        </div>
      </div>
    </main>
  );
}
