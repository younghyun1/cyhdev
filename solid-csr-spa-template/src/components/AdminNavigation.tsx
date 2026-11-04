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
      <li class={desktop() ? "site-nav-cell" : undefined}>
        <a
          href={ADMIN_DEFAULT_HREF}
          class={desktop()
            ? ["site-nav-control", active() && "site-nav-control--active"]
            : [
              "block rounded-sm px-4 py-2 font-mono text-sm transition-colors",
              active()
                ? "border-s-2 border-accent bg-surface-2 text-ink"
                : "text-ink-muted hover:bg-surface-2 hover:text-ink",
            ]}
          aria-current={active() ? "page" : undefined}
          onClick={() => props.onNavigate?.()}
        >
          <span class={desktop() ? "site-nav-label" : undefined}>{t("top_bar.admin.title")}</span>
        </a>
      </li>
    </Show>
  );
};

export default AdminNavigation;
