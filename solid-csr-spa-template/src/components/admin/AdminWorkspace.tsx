import { type Component, createEffect, For, type ParentComponent, Show } from "solid-js";
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
  let mobileNavigation: HTMLElement | undefined;
  createEffect(
    () => [location.pathname, location.hash, authorized()] as const,
    () => {
      const navigation = mobileNavigation;
      const active = navigation?.querySelector<HTMLElement>('[aria-current="location"]')
        ?? navigation?.querySelector<HTMLElement>('[aria-current="page"]');
      if (!navigation || !active) return;
      // Reveal the current tab without moving the document away from its fragment.
      navigation.scrollLeft += active.getBoundingClientRect().left
        - navigation.getBoundingClientRect().left - 12;
    },
  );

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
            ref={mobileNavigation}
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
