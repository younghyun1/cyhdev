import { Show, type ParentComponent } from "solid-js";
import { isAuthenticated, user } from "../state/auth";
import { t } from "../state/i18n";
import LanguageSelect from "./LanguageSelect";
import { MobileDialog } from "./MobileDialog";
import SystemStatusDetails from "./SystemStatusDetails";
import ThemeToggle from "./ThemeToggle";

type MobileDrawerProps = {
  readonly onClose: () => void;
  readonly onLogout: () => void;
};

const MobileDrawer: ParentComponent<MobileDrawerProps> = (props) => (
  <MobileDialog
    onClose={props.onClose}
    overlayClass="mobile-drawer-overlay md:hidden"
    panelClass="mobile-drawer"
    ariaLabelledBy="mobile-navigation-title"
    initialFocusSelector="[data-drawer-close]"
  >
    <header class="mobile-drawer-header">
      <span id="mobile-navigation-title" class="font-mono font-bold text-lg">
        {t("top_bar.menu.title")}
      </span>
      <button
        data-drawer-close
        type="button"
        onClick={() => props.onClose()}
        class="mobile-drawer-close"
        aria-label={t("common.close")}
      >
        <svg
          aria-hidden="true"
          class="h-6 w-6"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      </button>
    </header>
    <div class="mobile-drawer-scroll">
      <nav aria-label={t("top_bar.menu.title")}>
        <ul class="mobile-drawer-navigation">{props.children}</ul>
      </nav>
      <section class="mobile-drawer-section">
        <div class="mobile-drawer-control-row">
          <span>{t("top_bar.aria.toggle_theme")}</span>
          <ThemeToggle />
        </div>
        <div class="mobile-drawer-control-row">
          <span>{t("top_bar.language.label")}</span>
          <LanguageSelect />
        </div>
      </section>
      <section class="mobile-drawer-section">
        <Show
          when={isAuthenticated()}
          fallback={
            <a
              class="mobile-drawer-action"
              href="/login"
              onClick={() => props.onClose()}
            >
              <span class="mobile-drawer-status mobile-drawer-status-offline" />
              {t("top_bar.auth.login")}
            </a>
          }
        >
          <div class="mobile-drawer-identity">
            <img
              src={
                user()?.user_profile_picture?.user_profile_picture_link ||
                "/default-profile.png"
              }
              alt={t("profile.picture_alt")}
              width="40"
              height="40"
            />
            <span class="min-w-0">
              <strong>{user()?.user_info?.user_name}</strong>
              <small>{user()?.user_info?.user_email}</small>
            </span>
          </div>
          <a
            class="mobile-drawer-action"
            href="/edit-profile"
            onClick={() => props.onClose()}
          >
            {t("top_bar.profile.edit")}
          </a>
          <button
            type="button"
            class="mobile-drawer-action"
            onClick={() => props.onLogout()}
          >
            {t("top_bar.auth.logout")}
          </button>
        </Show>
      </section>
      <section class="mobile-drawer-section" aria-label={t("bottom_bar.site_status")}>
        <h2 class="mobile-drawer-section-title">{t("bottom_bar.site_status")}</h2>
        <SystemStatusDetails />
      </section>
    </div>
  </MobileDialog>
);

export default MobileDrawer;
