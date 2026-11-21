import { For, type ParentComponent } from "solid-js";
import { useLocation } from "@solidjs/router";

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

const TABS: ReadonlyArray<{
  readonly section: ArchiveSection;
  readonly href: string;
  readonly labelKey: UiTextKey;
}> = [
  { section: "search", href: ARCHIVE_PATH, labelKey: "kcna.nav.search" },
  { section: "trends", href: ARCHIVE_TRENDS_PATH, labelKey: "kcna.nav.trends" },
  { section: "status", href: ARCHIVE_STATUS_PATH, labelKey: "kcna.nav.status" },
];

/** The archive section's frame: title, tabs, and the active tab's page. */
const KcnaArchiveLayout: ParentComponent = (props) => {
  const location = useLocation();
  const section = () => sectionOf(location.pathname);

  return (
    <main class={`${pageStyles.page} kcna-page`}>
      <div class="kcna-shell">
        <header class="kcna-header">
          <div class="kcna-header-text">
            <h1 class={`${pageStyles.titleSm} kcna-heading`}>{t("kcna.title")}</h1>
            <p class="kcna-subtitle">{t("kcna.description")}</p>
          </div>
          <span class="kcna-badge">{t("kcna.admin_only")}</span>
        </header>
        <nav class="kcna-tabs" aria-label={t("kcna.title")}>
          <ul class="kcna-tab-list">
            <For each={TABS}>
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
