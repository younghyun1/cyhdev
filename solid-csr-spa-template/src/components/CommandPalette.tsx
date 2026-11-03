import { For, Show, createMemo, createSignal } from "solid-js";
import type { JSX } from "@solidjs/web";
import { useNavigate } from "@solidjs/router";
import type { UiTextKey } from "../i18n/keys";
import { ACCENTS, accent, setAccent } from "../state/accent";
import { t, tx } from "../state/i18n";
import { theme, toggleThemeFrom } from "../state/theme";
import { rankByQuery } from "../utils/fuzzyMatch";
import { MobileDialog } from "./MobileDialog";
import { NAV_LINKS } from "./PublicNavigation";
import "../styles/command-palette.css";

type Group = "pages" | "sections" | "actions";

type PaletteEntry = {
  readonly id: string;
  readonly group: Group;
  readonly label: string;
  readonly hint: string;
  readonly current?: boolean;
  readonly run: () => void;
};

const GROUP_LABELS: Readonly<Record<Group, UiTextKey>> = {
  pages: "command_palette.group.pages",
  sections: "command_palette.group.sections",
  actions: "command_palette.group.actions",
};

const SECTION_LINKS: ReadonlyArray<{
  readonly href: string;
  readonly parentKey: UiTextKey;
  readonly labelKey: UiTextKey;
}> = [
  { href: "/about/experience", parentKey: "top_bar.nav.about", labelKey: "about.nav.experience" },
  { href: "/about/projects", parentKey: "top_bar.nav.about", labelKey: "about.nav.projects" },
  { href: "/about/background", parentKey: "top_bar.nav.about", labelKey: "about.nav.background" },
  { href: "/about-blog/infrastructure", parentKey: "top_bar.nav.about_blog", labelKey: "about_blog.nav.infrastructure" },
  { href: "/about-blog/backend", parentKey: "top_bar.nav.about_blog", labelKey: "about_blog.nav.backend" },
  { href: "/about-blog/frontend", parentKey: "top_bar.nav.about_blog", labelKey: "about_blog.nav.frontend" },
  { href: "/about-blog/subsystems", parentKey: "top_bar.nav.about_blog", labelKey: "about_blog.nav.subsystems" },
  { href: "/about-blog/performance", parentKey: "top_bar.nav.about_blog", labelKey: "about_blog.nav.performance" },
];

const ACCENT_LABELS: Readonly<Record<(typeof ACCENTS)[number], UiTextKey>> = {
  amber: "accent.amber",
  lagoon: "accent.lagoon",
  cobalt: "accent.cobalt",
  orchid: "accent.orchid",
  graphite: "accent.graphite",
};

/**
 * Site-wide command palette: fuzzy search over public pages, About subpages,
 * and display actions. Implements the ARIA combobox pattern with a listbox;
 * focus containment, Escape, and inert background come from MobileDialog.
 */
export default function CommandPalette(props: { onClose: () => void }): JSX.Element {
  const navigate = useNavigate();
  const [query, setQuery] = createSignal("");
  const [activeIndex, setActiveIndex] = createSignal(0);
  let list: HTMLUListElement | undefined;

  const go = (href: string) => () => {
    props.onClose();
    navigate(href);
  };

  const entries = createMemo((): PaletteEntry[] => {
    const pages = NAV_LINKS.map(
      (link): PaletteEntry => ({
        id: `page:${link.href}`,
        group: "pages",
        label: t(link.labelKey),
        hint: link.href,
        run: go(link.href),
      }),
    );
    const sections = SECTION_LINKS.map(
      (link): PaletteEntry => ({
        id: `section:${link.href}`,
        group: "sections",
        label: `${t(link.parentKey)} › ${t(link.labelKey)}`,
        hint: link.href,
        run: go(link.href),
      }),
    );
    const themeAction: PaletteEntry = {
      id: "action:theme",
      group: "actions",
      label: t(
        theme() === "dark"
          ? "command_palette.action.theme_light"
          : "command_palette.action.theme_dark",
      ),
      hint: theme() === "dark" ? "☀" : "☾",
      run: () => {
        props.onClose();
        // Let the dialog unmount first so the transition snapshots the page.
        requestAnimationFrame(() =>
          toggleThemeFrom({ x: window.innerWidth / 2, y: window.innerHeight / 3 }),
        );
      },
    };
    const accents = ACCENTS.map(
      (name): PaletteEntry => ({
        id: `action:accent:${name}`,
        group: "actions",
        label: tx("command_palette.action.accent", { name: t(ACCENT_LABELS[name]) }),
        hint: name === accent() ? "●" : "○",
        current: name === accent(),
        run: () => setAccent(name),
      }),
    );
    return [...pages, ...sections, themeAction, ...accents];
  });

  const results = createMemo(() =>
    rankByQuery(entries(), query(), (entry) => [entry.label, entry.hint]),
  );
  const grouped = createMemo(() => {
    const groups: Array<{ group: Group; items: Array<{ entry: PaletteEntry; index: number }> }> = [];
    results().forEach((entry, index) => {
      const last = groups.at(-1);
      if (last && last.group === entry.group) last.items.push({ entry, index });
      else groups.push({ group: entry.group, items: [{ entry, index }] });
    });
    // With a query, results are ranked; without one, keep the group order.
    return groups;
  });
  const active = () => results()[Math.min(activeIndex(), results().length - 1)];

  const moveTo = (index: number) => {
    const count = results().length;
    if (count === 0) return;
    const next = ((index % count) + count) % count;
    setActiveIndex(next);
    queueMicrotask(() =>
      list
        ?.querySelector<HTMLElement>(`[data-index="${next}"]`)
        ?.scrollIntoView({ block: "nearest" }),
    );
  };

  const onKeyDown = (event: KeyboardEvent) => {
    switch (event.key) {
      case "ArrowDown":
        event.preventDefault();
        moveTo(activeIndex() + 1);
        break;
      case "ArrowUp":
        event.preventDefault();
        moveTo(activeIndex() - 1);
        break;
      case "Home":
        if (event.ctrlKey) {
          event.preventDefault();
          moveTo(0);
        }
        break;
      case "End":
        if (event.ctrlKey) {
          event.preventDefault();
          moveTo(results().length - 1);
        }
        break;
      case "Enter": {
        const entry = active();
        if (entry) {
          event.preventDefault();
          entry.run();
        }
        break;
      }
    }
  };

  return (
    <MobileDialog
      onClose={props.onClose}
      overlayClass="cmdk-overlay"
      panelClass="cmdk-panel"
      ariaLabel={t("command_palette.title")}
      initialFocusSelector=".cmdk-input"
    >
      <div class="cmdk-input-row">
        <span class="cmdk-prompt" aria-hidden="true">
          ›
        </span>
        <input
          class="cmdk-input"
          type="text"
          role="combobox"
          aria-expanded="true"
          aria-controls="cmdk-listbox"
          aria-autocomplete="list"
          aria-label={t("command_palette.open")}
          aria-activedescendant={active() ? `cmdk-option-${activeIndex()}` : undefined}
          placeholder={t("command_palette.placeholder")}
          autocomplete="off"
          spellcheck="false"
          value={query()}
          onInput={(event) => {
            setQuery(event.currentTarget.value);
            setActiveIndex(0);
          }}
          onKeyDown={onKeyDown}
        />
        <kbd class="cmdk-kbd">Esc</kbd>
      </div>
      <ul
        id="cmdk-listbox"
        class="cmdk-list"
        role="listbox"
        aria-label={t("command_palette.title")}
        ref={(element) => (list = element)}
      >
        <Show
          when={results().length > 0}
          fallback={<li class="cmdk-empty" role="presentation">{t("command_palette.empty")}</li>}
        >
          <For each={grouped()}>
            {(group) => (
              <li role="presentation">
                <p class="cmdk-group" id={`cmdk-group-${group.group}`}>
                  {t(GROUP_LABELS[group.group])}
                </p>
                <ul role="group" aria-labelledby={`cmdk-group-${group.group}`} class="cmdk-group-list">
                  <For each={group.items}>
                    {(item) => (
                      <li
                        id={`cmdk-option-${item.index}`}
                        data-index={item.index}
                        role="option"
                        aria-selected={item.index === activeIndex() ? "true" : "false"}
                        class="cmdk-option"
                        data-current={item.entry.current ? "" : undefined}
                        onPointerMove={() => {
                          if (activeIndex() !== item.index) setActiveIndex(item.index);
                        }}
                        onClick={() => item.entry.run()}
                      >
                        <span class="cmdk-option-label">{item.entry.label}</span>
                        <span class="cmdk-option-hint">{item.entry.hint}</span>
                      </li>
                    )}
                  </For>
                </ul>
              </li>
            )}
          </For>
        </Show>
      </ul>
      <p class="cmdk-footer">{t("command_palette.hint")}</p>
    </MobileDialog>
  );
}
