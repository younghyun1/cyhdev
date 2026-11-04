import { Show, createEffect, createSignal, onCleanup } from "solid-js";
import { useLocation } from "@solidjs/router";
import {
  isAuthenticated,
  setSuperuser,
  user,
  setAuthenticated,
  setUser,
} from "../state/auth";
import { authApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import LanguageSelect from "./LanguageSelect";
import ThemeToggle from "./ThemeToggle";
import AppearanceMenu from "./AppearanceMenu";
import CommandPaletteButton from "./CommandPaletteButton";
import { t } from "../state/i18n";
import AdminNavigation from "./AdminNavigation";
import MobileDrawer from "./MobileDrawer";
import PublicNavigation, { NAV_LINKS } from "./PublicNavigation";
import { createMediaQuery } from "../utils/mediaQuery";

const TopBar = () => {
  const location = useLocation();
  const isMobile = createMediaQuery("(max-width: 767px)");
  const [menuOpen, setMenuOpen] = createSignal(false);
  const [sidebarOpen, setSidebarOpen] = createSignal(false);
  let activeClickOutsideHandler: ((event: MouseEvent) => void) | null = null;

  const removeClickOutsideHandler = () => {
    if (!activeClickOutsideHandler) return;
    window.removeEventListener("mousedown", activeClickOutsideHandler);
    activeClickOutsideHandler = null;
  };

  const handleMenuToggle = (event: MouseEvent) => {
    event.preventDefault();
    const willOpen = !menuOpen();
    setMenuOpen(willOpen);
    removeClickOutsideHandler();
    if (!willOpen) return;
    activeClickOutsideHandler = (outsideEvent: MouseEvent) => {
      if (
        !(outsideEvent.target as HTMLElement).closest(
          ".profile-menu, .menu-toggle",
        )
      ) {
        setMenuOpen(false);
        removeClickOutsideHandler();
      }
    };
    window.addEventListener("mousedown", activeClickOutsideHandler);
  };

  const handleLogout = async () => {
    try {
      await authApi.logout();
    } catch {
      // Local authority must still be cleared if the session is already gone.
    }
    setAuthenticated(false);
    setUser(null);
    setSuperuser(false);
    setMenuOpen(false);
    setSidebarOpen(false);
  };

  onCleanup(removeClickOutsideHandler);

  createEffect(
    () => location.pathname,
    () => {
      setSidebarOpen(false);
      setMenuOpen(false);
      removeClickOutsideHandler();
    },
  );

  createEffect(
    () => isMobile(),
    (mobile) => {
      if (!mobile) setSidebarOpen(false);
    },
  );

  const isActive = (href: string) => {
    const pathname = location.pathname || "/";
    if (href === "/") return pathname === "/";
    return pathname === href || pathname.startsWith(`${href}/`);
  };

  const titleFromPath = () => {
    const pathname = location.pathname || "/";
    if (pathname === "/") return t("top_bar.nav.home");
    const segment = pathname.replace(/^\/+/, "").split("/")[0] || "";
    const match = NAV_LINKS.find((link) => link.href === `/${segment}`);
    if (match) return t(match.labelKey);
    return segment
      .split("-")
      .filter(Boolean)
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(" ");
  };

  return (
    <>
      <header
        data-site-bar="top"
        class="site-header site-header-glass fixed inset-x-0 top-0 z-40 text-ink transition-colors duration-90"
      >
        <div class="w-full px-3 sm:px-4 lg:px-6">
          <div class="site-header-row flex items-center justify-between gap-2 py-2 sm:py-3">
            <div class="site-header-leading flex min-w-0 flex-1 items-center gap-3 sm:gap-6">
              <button
                id="mobile-navigation-trigger"
                type="button"
                class="site-header-menu-button md:hidden p-1 text-ink-muted hover:text-ink hover:bg-surface-2 rounded-sm"
                onClick={() => setSidebarOpen(true)}
                aria-label={t("top_bar.aria.open_sidebar")}
                aria-haspopup="dialog"
                aria-expanded={sidebarOpen() ? "true" : "false"}
              >
                <svg
                  class="w-6 h-6"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  viewBox="0 0 24 24"
                  aria-hidden="true"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M4 6h16M4 12h16M4 18h16"
                  />
                </svg>
              </button>

              <a
                href="/"
                class="site-header-brand min-w-0 shrink-0 font-mono text-lg sm:text-xl md:text-2xl font-bold tracking-tight whitespace-nowrap"
              >
                <span class="site-header-mobile-title block md:hidden">
                  {titleFromPath()}
                </span>
                <span class="hidden md:block">{t("top_bar.site_title")}</span>
              </a>

              <nav class="site-header-navigation hidden md:block flex-1 md:overflow-visible">
                <ul class="flex items-center">
                  <PublicNavigation variant="desktop" isActive={isActive} />
                  <AdminNavigation variant="desktop" isActive={isActive} />
                </ul>
              </nav>
            </div>

            {/* Phones keep every other control in the drawer. */}
            <div class="site-header-mobile-actions flex shrink-0 items-center md:hidden">
              <Show
                when={isAuthenticated()}
                fallback={
                  <a href="/login" class="site-header-mobile-login">
                    <span class="site-nav-label">{t("top_bar.auth.login")}</span>
                  </a>
                }
              >
                <a
                  href="/edit-profile"
                  class="site-header-mobile-avatar"
                  aria-label={t("top_bar.profile.edit")}
                >
                  <img
                    src={
                      user()?.user_profile_picture?.user_profile_picture_link ||
                      "/default-profile.png"
                    }
                    alt=""
                    width="32"
                    height="32"
                  />
                </a>
              </Show>
            </div>

            <div class="site-header-actions flex shrink-0 items-center gap-2">
              <CommandPaletteButton />
              <ThemeToggle />
              <AppearanceMenu />
              <LanguageSelect />
              <Show
                when={isAuthenticated()}
                fallback={
                  <a
                    href="/login"
                    class={`${pageStyles.buttonSecondary} site-header-login`}
                  >
                    <span class="site-nav-label">{t("top_bar.auth.login")}</span>
                  </a>
                }
              >
                <div class="flex items-center gap-2 sm:gap-3">
                  <div class="site-header-identity hidden sm:flex flex-col items-end select-none" dir="auto">
                    <span class="font-medium text-xs sm:text-sm">
                      {user()?.user_info?.user_name}
                    </span>
                    <span class="text-[10px] sm:text-xs text-ink-faint">
                      {user()?.user_info?.user_email}
                    </span>
                  </div>
                  <div class="relative">
                    <button
                      class="menu-toggle profile-picture"
                      aria-label={t("top_bar.aria.open_user_menu")}
                      aria-haspopup="menu"
                      aria-expanded={menuOpen() ? "true" : "false"}
                      onClick={handleMenuToggle}
                    >
                      <img
                        src={
                          user()?.user_profile_picture
                            ?.user_profile_picture_link || "/default-profile.png"
                        }
                        alt={t("profile.picture_alt")}
                        width="40"
                        height="40"
                        class="w-8 h-8 sm:w-10 sm:h-10 rounded-full border-2 border-line object-cover transition ring-2 ring-transparent hover:ring-accent"
                      />
                    </button>
                    <Show when={menuOpen()}>
                      <div
                        class="profile-menu absolute end-0 mt-2 w-48 bg-surface/95 text-ink rounded-sm shadow-lg py-1 z-50 border border-line transition-colors duration-90"
                        role="menu"
                      >
                        <a
                          href="/edit-profile"
                          role="menuitem"
                          class="w-full text-start px-3 py-2 sm:px-4 sm:py-2 hover:bg-surface-2 rounded-sm flex items-center gap-2 text-xs sm:text-sm transition-colors duration-90"
                        >
                          <svg
                            width="18"
                            height="18"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            viewBox="0 0 24 24"
                            aria-hidden="true"
                          >
                            <path d="M12 20h9" />
                            <path d="M16.5 3.5a2.121 2.121 0 113 3L7 19l-4 1 1-4 12.5-12.5z" />
                          </svg>
                          {t("top_bar.profile.edit")}
                        </a>
                        <button
                          type="button"
                          role="menuitem"
                          class="w-full text-start px-3 py-2 sm:px-4 sm:py-2 hover:bg-surface-2 rounded-sm flex items-center gap-2 text-xs sm:text-sm transition-colors duration-90"
                          onClick={() => void handleLogout()}
                        >
                          <svg
                            width="18"
                            height="18"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            viewBox="0 0 24 24"
                            aria-hidden="true"
                          >
                            <path d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a2 2 0 01-2 2h-3a2 2 0 01-2-2V7a2 2 0 012-2h3a2 2 0 012 2v1" />
                          </svg>
                          {t("top_bar.auth.logout")}
                        </button>
                      </div>
                    </Show>
                  </div>
                </div>
              </Show>
            </div>
          </div>
        </div>
      </header>

      <Show when={sidebarOpen()}>
        <MobileDrawer
          onClose={() => setSidebarOpen(false)}
          onLogout={() => void handleLogout()}
        >
          <PublicNavigation
            variant="mobile"
            isActive={isActive}
            onNavigate={() => setSidebarOpen(false)}
          />
          <AdminNavigation
            variant="mobile"
            isActive={isActive}
            onNavigate={() => setSidebarOpen(false)}
          />
        </MobileDrawer>
      </Show>
    </>
  );
};

export default TopBar;
