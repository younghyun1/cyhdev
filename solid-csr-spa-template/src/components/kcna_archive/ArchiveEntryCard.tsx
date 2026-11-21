import { Show, type Component } from "solid-js";

import { t } from "../../state/i18n";
import { ARCHIVE_PATH } from "../../utils/kcna_archive/routes";
import { canViewArchive } from "./access";
import "../../styles/kcna-archive-entry.css";

/** The Projects page entry to the archive; it renders nothing for anyone but superusers. */
const ArchiveEntryCard: Component = () => (
  <Show when={canViewArchive()}>
    <a class="kcna-entry" href={ARCHIVE_PATH}>
      <span class="kcna-entry-text">
        <span class="kcna-entry-title">{t("kcna.title")}</span>
        <span class="kcna-entry-description">{t("kcna.description")}</span>
      </span>
      <span class="kcna-badge">{t("kcna.admin_only")}</span>
    </a>
  </Show>
);

export default ArchiveEntryCard;
