import AccountDeletionPanel from "../components/account/AccountDeletionPanel";
import OidcAccountPanel from "../components/account/OidcAccountPanel";
import ProfilePicturePanel from "../components/account/ProfilePicturePanel";
import ProfileSummaryPanel from "../components/account/ProfileSummaryPanel";
import { t } from "../state/i18n";
import { pageStyles } from "../styles/pageStyles";

function EditProfilePage() {
  return (
    <main class={pageStyles.page}>
      <div class={pageStyles.pageInnerNarrow}>
        <h1 class={`${pageStyles.title} mb-6`}>
          {t("page.edit_profile.title")}
        </h1>
        <div class="space-y-6">
          <ProfilePicturePanel />
          <ProfileSummaryPanel />
          <OidcAccountPanel />
          <AccountDeletionPanel />
        </div>
      </div>
    </main>
  );
}

export default EditProfilePage;
