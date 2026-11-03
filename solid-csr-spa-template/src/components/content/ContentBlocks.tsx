import { For, Show } from "solid-js";
import type { JSX } from "@solidjs/web";
import type {
  BarItem,
  ContentBlock,
  LinkCard,
  Role,
  StatItem,
} from "../../pages/about-locales/content_types";
import InlineText from "./InlineText";

function isExternal(href: string): boolean {
  return !href.startsWith("/");
}

/**
 * Pointer position for the card spotlight. A single delegated listener per
 * grid writes two custom properties on the hovered card; the gradient itself
 * is CSS, so nothing re-renders while the pointer moves.
 */
function trackSpotlight(event: PointerEvent): void {
  if (event.pointerType !== "mouse") return;
  const target = event.target instanceof Element ? event.target : null;
  const card = target?.closest<HTMLElement>(".cp-card");
  if (!card) return;
  const rect = card.getBoundingClientRect();
  card.style.setProperty("--cp-mx", `${event.clientX - rect.left}px`);
  card.style.setProperty("--cp-my", `${event.clientY - rect.top}px`);
}

function Stat(props: { item: StatItem }): JSX.Element {
  return (
    <div class="cp-stat">
      <dt class="cp-stat-value">
        <span class="sr-only">{props.item.value}</span>
        <Show
          when={props.item.count !== undefined}
          fallback={<span aria-hidden="true">{props.item.value}</span>}
        >
          <span aria-hidden="true">
            {props.item.prefix}
            <span
              class="cp-stat-count"
              style={{ "--cp-count-target": String(props.item.count ?? 0) }}
            />
            {props.item.suffix}
          </span>
        </Show>
      </dt>
      <dd class="cp-stat-label">
        <InlineText text={props.item.label} />
      </dd>
    </div>
  );
}

function Tags(props: { items: ReadonlyArray<string> }): JSX.Element {
  return (
    <ul class="cp-tags">
      <For each={props.items}>{(item) => <li class="cp-tag">{item}</li>}</For>
    </ul>
  );
}

function RoleEntry(props: { role: Role }): JSX.Element {
  return (
    <li class="cp-role cp-reveal">
      <span class="cp-role-node" aria-hidden="true" />
      <article class="cp-role-card">
        <header class="cp-role-header">
          <div class="cp-role-heading">
            <h3 class="cp-role-org">{props.role.organization}</h3>
            <p class="cp-role-title">{props.role.title}</p>
          </div>
          <p class="cp-role-meta">
            <span class="cp-role-period">{props.role.period}</span>
            <span class="cp-role-location">{props.role.location}</span>
          </p>
        </header>
        <Show when={props.role.summary}>
          {(summary) => (
            <p class="cp-role-summary">
              <InlineText text={summary()} />
            </p>
          )}
        </Show>
        <Show when={props.role.highlights.length > 0}>
          <ul class="cp-bullets">
            <For each={props.role.highlights}>
              {(item) => (
                <li>
                  <InlineText text={item} />
                </li>
              )}
            </For>
          </ul>
        </Show>
        <For each={props.role.projects ?? []}>
          {(project) => (
            <section class="cp-role-project">
              <h4 class="cp-role-project-name">{project.name}</h4>
              <ul class="cp-bullets">
                <For each={project.highlights}>
                  {(item) => (
                    <li>
                      <InlineText text={item} />
                    </li>
                  )}
                </For>
              </ul>
            </section>
          )}
        </For>
        <Show when={props.role.tags}>{(tags) => <Tags items={tags()} />}</Show>
      </article>
    </li>
  );
}

function Card(props: { card: LinkCard }): JSX.Element {
  const body = () => (
    <>
      <Show when={props.card.meta}>
        {(meta) => <p class="cp-card-meta">{meta()}</p>}
      </Show>
      <h3 class="cp-card-title">
        <Show when={props.card.href} fallback={props.card.title}>
          {(href) => (
            <a
              href={href()}
              class="cp-card-link"
              target={isExternal(href()) ? "_blank" : undefined}
              rel={isExternal(href()) ? "noopener noreferrer" : undefined}
            >
              {props.card.title}
              <span class="cp-card-arrow" aria-hidden="true">
                {isExternal(href()) ? "↗" : "→"}
              </span>
            </a>
          )}
        </Show>
      </h3>
      <p class="cp-card-text">
        <InlineText text={props.card.text} />
      </p>
      <Show when={props.card.tags}>{(tags) => <Tags items={tags()} />}</Show>
    </>
  );
  return <li class="cp-card cp-reveal">{body()}</li>;
}

function Bars(props: {
  caption: string;
  groups: ReadonlyArray<{ readonly label: string; readonly items: ReadonlyArray<BarItem> }>;
}): JSX.Element {
  return (
    <figure class="cp-bars">
      <For each={props.groups}>
        {(group) => {
          const max = Math.max(...group.items.map((item) => item.value), 1);
          return (
            <div class="cp-bar-group">
              <p class="cp-bar-group-label">{group.label}</p>
              <ul class="cp-bar-list">
                <For each={group.items}>
                  {(item) => (
                    <li class={item.emphasis ? "cp-bar cp-bar-emphasis" : "cp-bar"}>
                      <span class="cp-bar-label">{item.label}</span>
                      <span class="cp-bar-track" aria-hidden="true">
                        <span
                          class="cp-bar-fill"
                          style={{ "--cp-ratio": (item.value / max).toFixed(4) }}
                        />
                      </span>
                      <span class="cp-bar-value">{item.display}</span>
                    </li>
                  )}
                </For>
              </ul>
            </div>
          );
        }}
      </For>
      <figcaption class="cp-figcaption">
        <InlineText text={props.caption} />
      </figcaption>
    </figure>
  );
}

/**
 * Content is static per locale and page, and the frame remounts when either
 * changes, so blocks render once from plain data rather than as reactive
 * components.
 */
function renderBlock(block: ContentBlock): JSX.Element {
  switch (block.kind) {
    case "paragraph":
      return (
        <p class="cp-paragraph">
          <InlineText text={block.text} />
        </p>
      );
    case "list":
      return (
        <ul class="cp-bullets cp-bullets-loose">
          <For each={block.items}>
            {(item) => (
              <li>
                <InlineText text={item} />
              </li>
            )}
          </For>
        </ul>
      );
    case "callout":
      return (
        <aside class="cp-callout cp-reveal">
          <Show when={block.title}>
            {(title) => <p class="cp-callout-title">{title()}</p>}
          </Show>
          <p>
            <InlineText text={block.text} />
          </p>
        </aside>
      );
    case "stats":
      return (
        <dl class="cp-stats">
          <For each={block.items}>{(item) => <Stat item={item} />}</For>
        </dl>
      );
    case "roles":
      return (
        <ol class="cp-timeline">
          <For each={block.roles}>{(role) => <RoleEntry role={role} />}</For>
        </ol>
      );
    case "cards":
      return (
        <ul class="cp-cards" onPointerMove={trackSpotlight}>
          <For each={block.cards}>{(card) => <Card card={card} />}</For>
        </ul>
      );
    case "pipeline":
      return (
        <ol class="cp-pipeline" style={{ "--cp-steps": String(block.steps.length) }}>
          <For each={block.steps}>
            {(step, index) => (
              <li class="cp-pipeline-step">
                <span class="cp-pipeline-index" aria-hidden="true">
                  {String(index() + 1).padStart(2, "0")}
                </span>
                <p class="cp-pipeline-label">{step.label}</p>
                <p class="cp-pipeline-detail">
                  <InlineText text={step.detail} />
                </p>
              </li>
            )}
          </For>
        </ol>
      );
    case "tags":
      return (
        <div class="cp-tag-groups">
          <For each={block.groups}>
            {(group) => (
              <div class="cp-tag-group">
                <p class="cp-tag-group-label">{group.label}</p>
                <Tags items={group.items} />
              </div>
            )}
          </For>
        </div>
      );
    case "bars":
      return <Bars caption={block.caption} groups={block.groups} />;
    case "table": {
      const numeric = new Set(block.numericColumns ?? []);
      return (
        <div class="cp-table-wrap">
          <table class="cp-table">
            <Show when={block.caption}>
              {(caption) => (
                <caption>
                  <InlineText text={caption()} />
                </caption>
              )}
            </Show>
            <thead>
              <tr>
                <For each={block.columns}>
                  {(column, index) => (
                    <th scope="col" class={numeric.has(index()) ? "cp-numeric" : undefined}>
                      {column}
                    </th>
                  )}
                </For>
              </tr>
            </thead>
            <tbody>
              <For each={block.rows}>
                {(row) => (
                  <tr>
                    <For each={row}>
                      {(cell, index) => (
                        <td class={numeric.has(index()) ? "cp-numeric" : undefined}>
                          <InlineText text={cell} />
                        </td>
                      )}
                    </For>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
        </div>
      );
    }
  }
}

export default function ContentBlocks(props: {
  blocks: ReadonlyArray<ContentBlock>;
}): JSX.Element {
  return <For each={props.blocks}>{renderBlock}</For>;
}
