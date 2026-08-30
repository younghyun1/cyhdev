import { For, Show, type Component } from "solid-js";

import type { UiTextKey } from "../i18n/keys";
import { isAuthenticated, isSuperuser } from "../state/auth";
import { t } from "../state/i18n";

type AdminNavigationLink = {
  readonly href: string;
  readonly labelKey: UiTextKey;
};

type AdminNavigationProps = {
  readonly variant: "desktop" | "mobile";
  readonly isActive: (href: string) => boolean;
  readonly onNavigate?: () => void;
};

export const ADMIN_NAVIGATION_LINKS = [
  {
    href: "/admin/authorization",
    labelKey: "top_bar.admin.authorization",
  },
  { href: "/blog/new", labelKey: "top_bar.admin.blog" },
  { href: "/photographs", labelKey: "top_bar.admin.photographs" },
  { href: "/projects", labelKey: "top_bar.admin.projects" },
  { href: "/forum", labelKey: "top_bar.admin.forum" },
  { href: "/swagger-ui", labelKey: "top_bar.admin.openapi" },
] as const satisfies ReadonlyArray<AdminNavigationLink>;

const canViewAdminNavigation = (): boolean =>
  isAuthenticated() === true && isSuperuser() === true;

const AdminNavigation: Component<AdminNavigationProps> = (props) => {
  const sectionActive = (): boolean =>
    ADMIN_NAVIGATION_LINKS.some((link) => props.isActive(link.href));

  const linkClass = (href: string, mobile: boolean): string => {
    if (mobile) {
      return [
        "block rounded-sm px-4 py-2 font-mono text-sm transition-colors",
        props.isActive(href)
          ? "border-l-2 border-accent bg-surface-2 text-ink"
          : "text-ink-muted hover:bg-surface-2 hover:text-ink",
      ].join(" ");
    }

    return [
      "block whitespace-nowrap px-4 py-2 text-sm transition-colors duration-90",
      props.isActive(href)
        ? "bg-surface-2 text-ink"
        : "text-ink-muted hover:bg-surface-2 hover:text-accent",
    ].join(" ");
  };

  return (
    <Show when={canViewAdminNavigation()}>
      <Show
        when={props.variant === "desktop"}
        fallback={
          <li
            class="mt-3 border-t border-line pt-3"
            aria-label={t("top_bar.admin.title")}
          >
            <span class="block px-4 pb-1 font-mono text-xs font-semibold uppercase tracking-wide text-ink-faint">
              {t("top_bar.admin.title")}
            </span>
            <ul class="space-y-1">
              <For each={ADMIN_NAVIGATION_LINKS}>
                {(link) => (
                  <li>
                    <a
                      href={link.href}
                      class={linkClass(link.href, true)}
                      onClick={props.onNavigate}
                    >
                      {t(link.labelKey)}
                    </a>
                  </li>
                )}
              </For>
            </ul>
          </li>
        }
      >
        <li class="relative px-2 py-1 md:px-3">
          <details
            class="group relative"
            aria-label={t("top_bar.admin.title")}
          >
            <summary
              class={[
                "flex cursor-pointer list-none items-center gap-1 whitespace-nowrap transition-colors duration-90 [&::-webkit-details-marker]:hidden",
                sectionActive()
                  ? "text-ink underline decoration-accent decoration-2 underline-offset-8"
                  : "text-ink-muted hover:text-accent hover:underline hover:decoration-accent/40 hover:underline-offset-8",
              ]}
            >
              {t("top_bar.admin.title")}
              <svg
                class="h-3 w-3 transition-transform group-open:rotate-180"
                viewBox="0 0 12 12"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                aria-hidden="true"
              >
                <path d="m2.5 4.5 3.5 3 3.5-3" />
              </svg>
            </summary>
            <ul class="absolute right-0 z-50 mt-3 w-60 rounded-sm border border-line bg-surface/95 py-1 font-sans shadow-lg backdrop-blur">
              <For each={ADMIN_NAVIGATION_LINKS}>
                {(link) => (
                  <li>
                    <a href={link.href} class={linkClass(link.href, false)}>
                      {t(link.labelKey)}
                    </a>
                  </li>
                )}
              </For>
            </ul>
          </details>
        </li>
      </Show>
    </Show>
  );
};

export default AdminNavigation;
