import HardPurgePanel from "../components/admin/operations/HardPurgePanel";
import AdminWorkspace from "../components/admin/AdminWorkspace";
import I18nSyncPanel from "../components/admin/operations/I18nSyncPanel";
import MediaCleanupPanel from "../components/admin/operations/MediaCleanupPanel";
import RetentionNotificationsPanel from "../components/admin/operations/RetentionNotificationsPanel";
import { t } from "../state/i18n";
import "../styles/admin-operations.css";
import "../styles/admin-operations-dialog.css";

export default function AdminOperationsPage() {
  return (
    <AdminWorkspace>
      <div class="operations-page">
        <div class="operations-page-inner">
          <header class="operations-page-header">
            <p class="operations-eyebrow">{t("operations.eyebrow")}</p>
            <h1>{t("page.operations.title")}</h1>
            <p>{t("operations.subtitle")}</p>
          </header>
          <RetentionNotificationsPanel />
          <MediaCleanupPanel />
          <I18nSyncPanel />
          <HardPurgePanel />
        </div>
      </div>
    </AdminWorkspace>
  );
}
