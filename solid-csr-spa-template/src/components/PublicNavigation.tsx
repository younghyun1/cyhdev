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
      { href: "/minecraft", labelKey: "top_bar.nav.minecraft" },
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
            class={props.variant === "desktop" ? "site-nav-cell" : ""}
          >
            <a
              href={item.link.href}
              aria-current={props.isActive(item.link.href) ? "page" : undefined}
              class={
                props.variant === "desktop"
                  ? [
                      "site-nav-control",
                      props.isActive(item.link.href) && "site-nav-control--active",
                    ]
                  : [
                      "block px-4 py-2 font-mono text-sm rounded-sm transition-colors",
                      props.isActive(item.link.href)
                        ? "text-ink bg-surface-2 border-s-2 border-accent"
                        : "text-ink-muted hover:text-ink hover:bg-surface-2",
                    ]
              }
              onClick={navigate}
            >
              <span class={props.variant === "desktop" ? "site-nav-label" : undefined}>
                {t(item.link.labelKey)}
              </span>
            </a>
          </li>
        ) : (
          <li
            data-public-navigation={navigationId}
            class={props.variant === "desktop" ? "site-nav-cell" : ""}
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
                      "site-nav-control site-nav-control--group",
                      item.links.some((link) => props.isActive(link.href)) && "site-nav-control--active",
                    ]
                  : [
                      "w-full cursor-pointer appearance-none border-0 bg-transparent px-4 py-2 text-start font-mono text-sm rounded-sm transition-colors after:ms-1 after:content-['▾']",
                      item.links.some((link) => props.isActive(link.href))
                        ? "text-ink bg-surface-2 border-s-2 border-accent"
                        : "text-ink-muted hover:text-ink hover:bg-surface-2",
                    ]
              }
            >
              <span class={props.variant === "desktop" ? "site-nav-label" : undefined}>
                {t(item.labelKey)}
              </span>
            </button>
            <Show when={openGroup() === item.labelKey}>
              <ul
                id={`${navigationId}-${item.labelKey}`}
                class={
                  props.variant === "desktop"
                    ? "site-nav-panel"
                    : "mt-1 space-y-1 ps-3"
                }
              >
                <For each={item.links}>
                  {(link) => (
                    <li>
                      <a
                        href={link.href}
                        aria-current={props.isActive(link.href) ? "page" : undefined}
                        class={[
                          "block truncate rounded-sm px-3 py-2 font-mono text-sm transition-colors",
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
