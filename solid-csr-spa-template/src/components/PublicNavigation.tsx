import {
  For,
  Show,
  type Component,
  createSignal,
  createUniqueId,
  onSettled,
} from "solid-js";

import type { UiTextKey } from "../i18n/keys";
import { t } from "../state/i18n";

type NavLink = {
  href: string;
  labelKey: UiTextKey;
};

type NavItem =
  | { kind: "link"; link: NavLink }
  | { kind: "group"; labelKey: UiTextKey; links: ReadonlyArray<NavLink> };

export const NAV_ITEMS: ReadonlyArray<NavItem> = [
  {
    kind: "link",
    link: { href: "/", labelKey: "top_bar.nav.home" },
  },
  {
    kind: "group",
    labelKey: "top_bar.nav.about_group",
    links: [
      { href: "/about", labelKey: "top_bar.nav.about" },
      { href: "/about-blog", labelKey: "top_bar.nav.about_blog" },
      { href: "/backend-stats", labelKey: "top_bar.nav.backend_stats" },
    ],
  },
  {
    kind: "link",
    link: { href: "/blog", labelKey: "top_bar.nav.blog" },
  },
  {
    kind: "group",
    labelKey: "top_bar.nav.community_group",
    links: [
      { href: "/forum", labelKey: "top_bar.nav.forum" },
      { href: "/live-chat", labelKey: "top_bar.nav.live_chat" },
      { href: "/visitor-board", labelKey: "top_bar.nav.visitor_board" },
    ],
  },
  {
    kind: "link",
    link: { href: "/photographs", labelKey: "top_bar.nav.photographs" },
  },
  {
    kind: "group",
    labelKey: "top_bar.nav.projects_group",
    links: [
      { href: "/projects", labelKey: "top_bar.nav.projects" },
      { href: "/geo-ip-db", labelKey: "top_bar.nav.geo_ip" },
      {
        href: "/eu5-locations-db",
        labelKey: "top_bar.nav.eu5_locations_db",
      },
    ],
  },
];

export const NAV_LINKS: ReadonlyArray<NavLink> = NAV_ITEMS.flatMap((item) =>
  item.kind === "link" ? [item.link] : item.links,
);

type PublicNavigationProps = {
  variant: "desktop" | "mobile";
  isActive: (href: string) => boolean;
  onNavigate?: () => void;
};

const PublicNavigation: Component<PublicNavigationProps> = (props) => {
  const navigationId = createUniqueId();
  const [openGroup, setOpenGroup] = createSignal<UiTextKey | null>(null);
  const close = () => setOpenGroup(null);
  const navigate = () => {
    close();
    props.onNavigate?.();
  };

  onSettled(() => {
    const closeOutside = (event: PointerEvent) => {
      const owner =
        event.target instanceof Element
          ? event.target.closest<HTMLElement>("[data-public-navigation]")
          : null;
      if (owner?.dataset.publicNavigation !== navigationId) close();
    };
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") close();
    };
    document.addEventListener("pointerdown", closeOutside);
    document.addEventListener("keydown", closeOnEscape);
    return () => {
      document.removeEventListener("pointerdown", closeOutside);
      document.removeEventListener("keydown", closeOnEscape);
    };
  });

  return (
    <For each={NAV_ITEMS}>
      {(item) =>
        item.kind === "link" ? (
          <li
            data-public-navigation={navigationId}
            class={props.variant === "desktop" ? "py-1 px-2 md:px-3" : ""}
          >
            <a
              href={item.link.href}
              aria-current={props.isActive(item.link.href) ? "page" : undefined}
              class={
                props.variant === "desktop"
                  ? [
                      "whitespace-nowrap transition-colors duration-90",
                      props.isActive(item.link.href)
                        ? "text-ink underline decoration-accent decoration-2 underline-offset-8"
                        : "text-ink-muted no-underline hover:text-accent hover:underline hover:decoration-accent/40 hover:underline-offset-8",
                    ]
                  : [
                      "block px-4 py-2 font-mono text-sm rounded-sm transition-colors",
                      props.isActive(item.link.href)
                        ? "text-ink bg-surface-2 border-l-2 border-accent"
                        : "text-ink-muted hover:text-ink hover:bg-surface-2",
                    ]
              }
              onClick={navigate}
            >
              {t(item.link.labelKey)}
            </a>
          </li>
        ) : (
          <li
            data-public-navigation={navigationId}
            class={props.variant === "desktop" ? "relative py-1 px-2 md:px-3" : ""}
          >
            <button
              type="button"
              aria-expanded={openGroup() === item.labelKey ? "true" : "false"}
              aria-controls={`${navigationId}-${item.labelKey}`}
              onClick={() => {
                setOpenGroup((current) =>
                  current === item.labelKey ? null : item.labelKey,
                );
              }}
              class={
                props.variant === "desktop"
                  ? [
                      "cursor-pointer appearance-none border-0 bg-transparent p-0 font-mono text-sm whitespace-nowrap transition-colors duration-90 after:ml-1 after:content-['▾']",
                      item.links.some((link) => props.isActive(link.href))
                        ? "text-ink underline decoration-accent decoration-2 underline-offset-8"
                        : "text-ink-muted hover:text-accent",
                    ]
                  : [
                      "w-full cursor-pointer appearance-none border-0 bg-transparent px-4 py-2 text-left font-mono text-sm rounded-sm transition-colors after:ml-1 after:content-['▾']",
                      item.links.some((link) => props.isActive(link.href))
                        ? "text-ink bg-surface-2 border-l-2 border-accent"
                        : "text-ink-muted hover:text-ink hover:bg-surface-2",
                    ]
              }
            >
              {t(item.labelKey)}
            </button>
            <Show when={openGroup() === item.labelKey}>
              <ul
                id={`${navigationId}-${item.labelKey}`}
                class={
                  props.variant === "desktop"
                    ? "absolute left-0 z-50 mt-2 min-w-52 rounded-sm border border-line bg-surface/98 p-1 shadow-xl"
                    : "mt-1 space-y-1 pl-3"
                }
              >
                <For each={item.links}>
                  {(link) => (
                    <li>
                      <a
                        href={link.href}
                        aria-current={props.isActive(link.href) ? "page" : undefined}
                        class={[
                          "block whitespace-nowrap rounded-sm px-3 py-2 font-mono text-sm transition-colors",
                          props.isActive(link.href)
                            ? "bg-surface-2 text-ink"
                            : "text-ink-muted hover:bg-surface-2 hover:text-ink",
                        ]}
                        onClick={navigate}
                      >
                        {t(link.labelKey)}
                      </a>
                    </li>
                  )}
                </For>
              </ul>
            </Show>
          </li>
        )
      }
    </For>
  );
};

export default PublicNavigation;
