import { Show, createSignal, For, onCleanup } from "solid-js";
import { useLocation } from "@solidjs/router";
import {
  isAuthenticated,
  setSuperuser,
  user,
  setAuthenticated,
  setUser,
} from "../state/auth";
import { theme, toggleTheme } from "../state/theme";
import { authApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import LanguageSelect from "./LanguageSelect";
import type { UiTextKey } from "../i18n/keys";
import { t } from "../state/i18n";

const [menuOpen, setMenuOpen] = createSignal(false);
const [sidebarOpen, setSidebarOpen] = createSignal(false);

let activeClickOutsideHandler: ((event: MouseEvent) => void) | null = null;

function removeClickOutsideHandler() {
  if (activeClickOutsideHandler) {
    window.removeEventListener("mousedown", activeClickOutsideHandler);
    activeClickOutsideHandler = null;
  }
}

const handleMenuToggle = (e: MouseEvent) => {
  e.preventDefault();
  const willOpen = !menuOpen();
  setMenuOpen(willOpen);

  removeClickOutsideHandler();

  if (willOpen) {
    activeClickOutsideHandler = (event: MouseEvent) => {
      if (
        !(event.target as HTMLElement).closest(".profile-menu, .menu-toggle")
      ) {
        setMenuOpen(false);
        removeClickOutsideHandler();
      }
    };
    window.addEventListener("mousedown", activeClickOutsideHandler);
  }
};

const handleLogout = async () => {
  try {
    await authApi.logout();
  } catch {
    // Ignore error; in either case we void the state.
  }
  setAuthenticated(false);
  setUser(null);
  setSuperuser(false);
  setMenuOpen(false);
};

type NavLink = {
  href: string;
  labelKey: UiTextKey;
};

const NAV_LINKS: NavLink[] = [
  { href: "/", labelKey: "top_bar.nav.home" },
  { href: "/about", labelKey: "top_bar.nav.about" },
  { href: "/about-blog", labelKey: "top_bar.nav.about_blog" },
  { href: "/blog", labelKey: "top_bar.nav.blog" },
  { href: "/photographs", labelKey: "top_bar.nav.photographs" },
  { href: "/live-chat", labelKey: "top_bar.nav.live_chat" },
  { href: "/projects", labelKey: "top_bar.nav.projects" },
  { href: "/visitor-board", labelKey: "top_bar.nav.visitor_board" },
  { href: "/geo-ip-db", labelKey: "top_bar.nav.geo_ip" },
  { href: "/backend-stats", labelKey: "top_bar.nav.backend_stats" },
];

const TopBar = () => {
  const location = useLocation();

  onCleanup(() => removeClickOutsideHandler());

  const titleFromPath = () => {
    const pathname = location.pathname || "/";
    if (pathname === "/") return t("top_bar.nav.home");

    const segment = pathname.replace(/^\/+/, "").split("/")[0] || "";

    const match = NAV_LINKS.find((l) => l.href === `/${segment}`);
    if (match) return t(match.labelKey);

    return segment
      .split("-")
      .filter(Boolean)
      .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
      .join(" ");
  };

  return (
    <>
      <header class="fixed top-0 left-0 right-0 z-40 border-b border-slate-200/80 dark:border-slate-800 bg-slate-50/80 dark:bg-slate-950/70 text-slate-900 dark:text-slate-100 backdrop-blur transition-colors duration-90">
        <div class="w-full px-3 sm:px-4 lg:px-6">
          <div class="flex items-center justify-between gap-2 py-2 sm:py-3">
            <div class="flex min-w-0 flex-1 items-center gap-3 sm:gap-6">
              {/* Hamburger Button (Mobile Only) */}
              <button
                class="md:hidden p-1 text-slate-700 dark:text-slate-200 focus:outline-none hover:bg-slate-100 dark:hover:bg-slate-800 rounded"
                onClick={() => setSidebarOpen(true)}
                aria-label={t("top_bar.aria.open_sidebar")}
              >
                <svg
                  class="w-6 h-6"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M4 6h16M4 12h16M4 18h16"
                  />
                </svg>
              </button>

              {/* Logo: Emoji on mobile, Text on desktop */}
              <a
                href="/"
                class="shrink-0 text-lg sm:text-xl md:text-2xl font-bold tracking-tight whitespace-nowrap"
              >
                <span class="block md:hidden text-2xl">{titleFromPath()}</span>
                <span class="hidden md:block">{t("top_bar.site_title")}</span>
              </a>

              {/* Nav: Hidden on mobile, inline on md+ */}
              <nav class="hidden md:block flex-1 overflow-x-auto md:overflow-visible ml-2">
                <ul class="flex items-center text-xs sm:text-sm md:text-base min-w-max md:min-w-0">
                  <For each={NAV_LINKS}>
                    {(link) => (
                      <li class="py-1 px-2 md:px-3">
                        <a
                          href={link.href}
                          class="whitespace-nowrap no-underline text-slate-600 dark:text-slate-300 hover:text-amber-700 dark:hover:text-amber-300 hover:underline transition-colors duration-90"
                        >
                          {t(link.labelKey)}
                        </a>
                      </li>
                    )}
                  </For>
                </ul>
              </nav>
            </div>

            {/* Right: auth / theme */}
            <div class="flex shrink-0 items-center gap-3 sm:gap-4">
              <Show
                when={isAuthenticated()}
                fallback={
                  <div class="flex items-center gap-2 sm:gap-3">
                    <button
                      type="button"
                      class="text-xs border rounded border-slate-300 dark:border-slate-700 bg-white/80 dark:bg-slate-900/70 text-slate-900 dark:text-slate-100 hover:bg-slate-100 dark:hover:bg-slate-800 transition-all duration-90 flex items-center justify-center gap-1 w-8 h-8 p-0"
                      aria-label={t("top_bar.aria.toggle_theme")}
                      onClick={toggleTheme}
                    >
                      <span class="inline-block transition-colors duration-90">
                        {theme() === "dark" ? "🌙" : "☀️"}
                      </span>
                    </button>

                    <LanguageSelect />

                    <span class="relative">
                      <span class="inline-block w-3 h-3 rounded-full bg-rose-500 shadow-[0_0_8px_2px_rgb(244,63,94,0.6)] mr-1 sm:mr-2" />
                    </span>

                    <a
                      href="/login"
                      class={`${pageStyles.buttonSecondary} px-3 py-1.5 sm:px-4 sm:py-2 text-xs sm:text-sm whitespace-nowrap`}
                    >
                      {t("top_bar.auth.login")}
                    </a>
                  </div>
                }
              >
                <div class="flex items-center gap-2 sm:gap-4">
                  <button
                    type="button"
                    class="text-xs border rounded border-slate-300 dark:border-slate-700 bg-white/80 dark:bg-slate-900/70 text-slate-900 dark:text-slate-100 hover:bg-slate-100 dark:hover:bg-slate-800 transition-all duration-90 flex items-center justify-center gap-1 w-8 h-8 p-0"
                    aria-label={t("top_bar.aria.toggle_theme")}
                    onClick={toggleTheme}
                  >
                    <span class="inline-block transition-colors duration-90">
                      {theme() === "dark" ? "🌙" : "☀️"}
                    </span>
                  </button>

                  <LanguageSelect />

                  <span class="relative flex items-center">
                    <span class="inline-block w-3 h-3 rounded-full bg-emerald-400 shadow-[0_0_8px_2px_rgb(52,211,153,0.7)] mr-1 sm:mr-2" />
                  </span>

                  <div class="hidden sm:flex flex-col items-end mr-1 sm:mr-2 select-none">
                    <span class="font-medium text-xs sm:text-sm">
                      {user()?.user_info?.user_name}
                    </span>

                    <span class="text-[10px] sm:text-xs text-slate-400">
                      {user()?.user_info?.user_email}
                    </span>
                  </div>

                  <div class="relative">
                    <button
                      class="menu-toggle profile-picture focus:outline-none"
                      aria-label={t("top_bar.aria.open_user_menu")}
                      aria-haspopup="menu"
                      aria-expanded={menuOpen() ? "true" : "false"}
                      tabIndex={0}
                      onClick={handleMenuToggle}
                    >
                      <img
                        src={
                          user()?.user_profile_picture
                            ?.user_profile_picture_link ||
                          "/default-profile.png"
                        }
                        alt="User"
                        class="w-8 h-8 sm:w-10 sm:h-10 rounded-full border-2 border-white shadow-md object-cover transition ring-2 ring-transparent hover:ring-amber-500"
                      />
                    </button>

                    <Show when={menuOpen()}>
                      <div class="profile-menu absolute right-0 mt-2 w-40 sm:w-48 bg-white/95 text-slate-900 rounded shadow-lg py-1 z-50 dark:bg-slate-950/95 dark:text-slate-100 border border-slate-200/80 dark:border-slate-800 transition-colors duration-90">
                        <a
                          href="/edit-profile"
                          class="w-full text-left px-3 py-2 sm:px-4 sm:py-2 hover:bg-slate-100 hover:dark:bg-slate-800 rounded flex items-center gap-2 text-xs sm:text-sm transition-colors duration-90"
                        >
                          <svg
                            width="18"
                            height="18"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            viewBox="0 0 24 24"
                          >
                            <path d="M12 20h9" />

                            <path d="M16.5 3.5a2.121 2.121 0 113 3L7 19l-4 1 1-4 12.5-12.5z" />
                          </svg>
                          {t("top_bar.profile.edit")}
                        </a>

                        <button
                          class="w-full text-left px-3 py-2 sm:px-4 sm:py-2 hover:bg-slate-100 hover:dark:bg-slate-800 rounded flex items-center gap-2 text-xs sm:text-sm transition-colors duration-90"
                          onClick={handleLogout}
                        >
                          <svg
                            width="18"
                            height="18"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            viewBox="0 0 24 24"
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

      {/* Mobile Sidebar (Drawer) */}
      <Show when={sidebarOpen()}>
        <div class="fixed inset-0 z-50 flex md:hidden">
          {/* Backdrop */}
          <div
            class="fixed inset-0 bg-black/50 transition-opacity"
            onClick={() => setSidebarOpen(false)}
          />

          {/* Sidebar */}
          <aside class="relative z-50 w-64 bg-white/95 dark:bg-slate-950/95 h-full shadow-xl flex flex-col transition-transform">
            <div class="p-4 border-b border-slate-200/80 dark:border-slate-800 flex items-center justify-between">
              <span class="font-bold text-lg text-slate-900 dark:text-slate-100">
                {t("top_bar.menu.title")}
              </span>
              <button
                onClick={() => setSidebarOpen(false)}
                class="text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200 focus:outline-none"
                aria-label={t("common.close")}
              >
                <svg
                  class="w-6 h-6"
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
            </div>
            <nav class="flex-1 overflow-y-auto py-4">
              <ul class="space-y-1 px-2">
                <For each={NAV_LINKS}>
                  {(link) => (
                    <li>
                      <a
                        href={link.href}
                        class="block px-4 py-2 text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 rounded transition-colors"
                        onClick={() => setSidebarOpen(false)}
                      >
                        {t(link.labelKey)}
                      </a>
                    </li>
                  )}
                </For>
              </ul>
            </nav>
          </aside>
        </div>
      </Show>
    </>
  );
};

export default TopBar;
