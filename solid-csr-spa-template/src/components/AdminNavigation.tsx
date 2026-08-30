import { type Component, Show } from "solid-js";

import {
  ADMIN_DEFAULT_HREF,
  ADMIN_TOP_BAR_ACTIVE_HREFS,
} from "./admin/navigation";
import { isAuthenticated, isSuperuser } from "../state/auth";
import { t } from "../state/i18n";

type AdminNavigationProps = {
  readonly variant: "desktop" | "mobile";
  readonly isActive: (href: string) => boolean;
  readonly onNavigate?: () => void;
};

export const canViewAdminNavigation = (): boolean =>
  isAuthenticated() === true && isSuperuser() === true;

const AdminNavigation: Component<AdminNavigationProps> = (props) => {
  const active = (): boolean =>
    ADMIN_TOP_BAR_ACTIVE_HREFS.some((href) => props.isActive(href));
  const desktop = () => props.variant === "desktop";

  return (
    <Show when={canViewAdminNavigation()}>
      <li class={desktop() ? "px-2 py-1 md:px-3" : undefined}>
        <a
          href={ADMIN_DEFAULT_HREF}
          class={desktop()
            ? [
              "whitespace-nowrap transition-colors duration-90",
              active()
                ? "text-ink underline decoration-accent decoration-2 underline-offset-8"
                : "text-ink-muted no-underline hover:text-accent hover:underline hover:decoration-accent/40 hover:underline-offset-8",
            ].join(" ")
            : [
              "block rounded-sm px-4 py-2 font-mono text-sm transition-colors",
              active()
                ? "border-l-2 border-accent bg-surface-2 text-ink"
                : "text-ink-muted hover:bg-surface-2 hover:text-ink",
            ].join(" ")}
          aria-current={active() ? "page" : undefined}
          onClick={() => props.onNavigate?.()}
        >
          {t("top_bar.admin.title")}
        </a>
      </li>
    </Show>
  );
};

export default AdminNavigation;
