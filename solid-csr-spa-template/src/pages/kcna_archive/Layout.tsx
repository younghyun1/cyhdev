import { For, Show, type ParentComponent } from "solid-js";
import { useLocation } from "@solidjs/router";

import { archiveIsRestricted, canManageArchive } from "../../components/kcna_archive/access";
import type { UiTextKey } from "../../i18n/keys";
import { t } from "../../state/i18n";
import { pageStyles } from "../../styles/pageStyles";
import {
  ARCHIVE_PATH,
  ARCHIVE_STATUS_PATH,
  ARCHIVE_TRENDS_PATH,
  sectionOf,
  type ArchiveSection,
} from "../../utils/kcna_archive/routes";
import "../../styles/kcna-archive-entry.css";
import "../../styles/kcna-archive.css";
import "../../styles/kcna-archive-reader.css";
import "../../styles/kcna-archive-charts.css";

interface Tab {
  readonly section: ArchiveSection;
  readonly href: string;
  readonly labelKey: UiTextKey;
}

const READER_TABS: ReadonlyArray<Tab> = [
  { section: "search", href: ARCHIVE_PATH, labelKey: "kcna.nav.search" },
  { section: "trends", href: ARCHIVE_TRENDS_PATH, labelKey: "kcna.nav.trends" },
];
/** The snapshot's status and reload are for whoever manages the archive. */
const STATUS_TAB: Tab = { section: "status", href: ARCHIVE_STATUS_PATH, labelKey: "kcna.nav.status" };

/** The archive section's frame: title, tabs, and the active tab's page. */
const KcnaArchiveLayout: ParentComponent = (props) => {
  const location = useLocation();
  const section = () => sectionOf(location.pathname);
  const tabs = (): ReadonlyArray<Tab> => (canManageArchive() ? [...READER_TABS, STATUS_TAB] : READER_TABS);

  return (
    <main class={`${pageStyles.page} kcna-page`}>
      <div class="kcna-shell">
        <header class="kcna-header">
          <div class="kcna-header-text">
            <h1 class={`${pageStyles.titleSm} kcna-heading`}>{t("kcna.title")}</h1>
            <p class="kcna-subtitle">{t("kcna.description")}</p>
            <p class="kcna-disclaimer">{t("kcna.disclaimer")}</p>
          </div>
          <Show when={archiveIsRestricted()}>
            <span class="kcna-badge">{t("kcna.admin_only")}</span>
          </Show>
        </header>
        <nav class="kcna-tabs" aria-label={t("kcna.title")}>
          <ul class="kcna-tab-list">
            <For each={tabs()}>
              {(tab) => (
                <li>
                  <a
                    class="kcna-tab"
                    href={tab.href}
                    title={t(tab.labelKey)}
                    aria-current={section() === tab.section ? "page" : undefined}
                  >
                    <span class="ui-clip">{t(tab.labelKey)}</span>
                  </a>
                </li>
              )}
            </For>
          </ul>
        </nav>
        {props.children}
      </div>
    </main>
  );
};

export default KcnaArchiveLayout;
