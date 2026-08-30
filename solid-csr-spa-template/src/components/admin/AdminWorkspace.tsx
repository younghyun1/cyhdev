import { type Component, For, type ParentComponent, Show } from "solid-js";
import { useLocation } from "@solidjs/router";

import { isAuthenticated, isSuperuser } from "../../state/auth";
import { t } from "../../state/i18n";
import {
  ADMIN_WORKSPACE_LINKS,
  isAdminWorkspaceLinkActive,
} from "./navigation";
import "../../styles/admin-workspace.css";

type NavigationLinksProps = {
  readonly pathname: string;
  readonly hash: string;
};

const AdminWorkspaceLinks: Component<NavigationLinksProps> = (props) => (
  <ul class="admin-workspace-navigation-list">
    <For each={ADMIN_WORKSPACE_LINKS}>
      {(link) => {
        const active = () =>
          isAdminWorkspaceLinkActive(link.href, props.pathname, props.hash);
        return (
          <li>
            <a
              href={link.href}
              rel={"external" in link ? "external" : undefined}
              class={{
                "admin-workspace-link": true,
                "admin-workspace-link-nested": link.depth === 1,
                "admin-workspace-link-active": active(),
              }}
              aria-current={active()
                ? (link.depth === 1 ? "location" : "page")
                : undefined}
            >
              {t(link.labelKey)}
            </a>
          </li>
        );
      }}
    </For>
  </ul>
);

const AdminWorkspace: ParentComponent = (props) => {
  const location = useLocation();
  const authorized = () => isAuthenticated() === true && isSuperuser() === true;

  return (
    <Show when={authorized()}>
      <div class="admin-workspace">
        <aside class="admin-workspace-sidebar">
          <div class="admin-workspace-sidebar-inner">
            <p class="admin-workspace-title">{t("top_bar.admin.title")}</p>
            <nav aria-label={t("top_bar.admin.title")}>
              <AdminWorkspaceLinks
                pathname={location.pathname}
                hash={location.hash}
              />
            </nav>
          </div>
        </aside>
        <div class="admin-workspace-main">
          <nav
            class="admin-workspace-mobile-navigation"
            aria-label={t("top_bar.admin.title")}
          >
            <AdminWorkspaceLinks
              pathname={location.pathname}
              hash={location.hash}
            />
          </nav>
          <div class="admin-workspace-content">{props.children}</div>
        </div>
      </div>
    </Show>
  );
};

export default AdminWorkspace;
