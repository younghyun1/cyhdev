import { Show, createSignal, For, onCleanup } from "solid-js";
import { useLocation } from "@solidjs/router";
import {
  isAuthenticated,
  isSuperuser,
  setSuperuser,
  user,
  setAuthenticated,
  setUser,
} from "../state/auth";
import { authApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import LanguageSelect from "./LanguageSelect";
import ThemeToggle from "./ThemeToggle";
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

  // Active-route check for nav links; exact match for "/", segment-prefix
  // match elsewhere so /blog/:id still highlights Blog but /about-blog does
  // not highlight /about.
  const isActive = (href: string) => {
    const pathname = location.pathname || "/";
    if (href === "/") return pathname === "/";
    return pathname === href || pathname.startsWith(`${href}/`);
  };

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
      <header class="fixed top-0 left-0 right-0 z-40 border-b border-line bg-paper/85 text-ink backdrop-blur transition-colors duration-90">
        <div class="w-full px-3 sm:px-4 lg:px-6">
          <div class="flex items-center justify-between gap-2 py-2 sm:py-3">
            <div class="flex min-w-0 flex-1 items-center gap-3 sm:gap-6">
              {/* Hamburger Button (Mobile Only) */}
              <button
                class="md:hidden p-1 text-ink-muted hover:text-ink hover:bg-surface-2 rounded-sm"
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
                class="shrink-0 font-mono text-lg sm:text-xl md:text-2xl font-bold tracking-tight whitespace-nowrap"
              >
                <span class="block md:hidden text-2xl">{titleFromPath()}</span>
                <span class="hidden md:block">{t("top_bar.site_title")}</span>
              </a>

              {/* Nav: Hidden on mobile, inline on md+ */}
              <nav class="hidden md:block flex-1 overflow-x-auto md:overflow-visible ml-2">
                <ul class="flex items-center font-mono text-sm min-w-max md:min-w-0">
                  <For each={NAV_LINKS}>
                    {(link) => (
                      <li class="py-1 px-2 md:px-3">
                        <a
                          href={link.href}
                          class={`whitespace-nowrap transition-colors duration-90 ${
                            isActive(link.href)
                              ? "text-ink underline decoration-accent decoration-2 underline-offset-8"
                              : "text-ink-muted no-underline hover:text-accent hover:underline hover:decoration-accent/40 hover:underline-offset-8"
                          }`}
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
                    <ThemeToggle />

                    <LanguageSelect />

                    <span class="relative">
                      <span class="inline-block w-3 h-3 rounded-full bg-danger shadow-[0_0_8px_2px_var(--glow-danger)] mr-1 sm:mr-2" />
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
                  <ThemeToggle />

                  <LanguageSelect />

                  <span class="relative flex items-center">
                    <span class="inline-block w-3 h-3 rounded-full bg-ok shadow-[0_0_8px_2px_var(--glow-ok)] mr-1 sm:mr-2" />
                  </span>

                  <div class="hidden sm:flex flex-col items-end mr-1 sm:mr-2 select-none">
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
                      tabindex={0}
                      onClick={handleMenuToggle}
                    >
                      <img
                        src={
                          user()?.user_profile_picture
                            ?.user_profile_picture_link ||
                          "/default-profile.png"
                        }
                        alt={t("profile.picture_alt")}
                        class="w-8 h-8 sm:w-10 sm:h-10 rounded-full border-2 border-line object-cover transition ring-2 ring-transparent hover:ring-accent"
                      />
                    </button>

                    <Show when={menuOpen()}>
                      <div class="profile-menu absolute right-0 mt-2 w-40 sm:w-48 bg-surface/95 text-ink rounded-sm shadow-lg py-1 z-50 border border-line transition-colors duration-90">
                        <a
                          href="/edit-profile"
                          class="w-full text-left px-3 py-2 sm:px-4 sm:py-2 hover:bg-surface-2 rounded-sm flex items-center gap-2 text-xs sm:text-sm transition-colors duration-90"
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

                        <Show when={isSuperuser() === true}>
                          <a
                            href="/admin/authorization"
                            class="w-full text-left px-3 py-2 sm:px-4 sm:py-2 hover:bg-surface-2 rounded-sm flex items-center gap-2 text-xs sm:text-sm transition-colors duration-90"
                          >
                            {t("top_bar.profile.authorization")}
                          </a>
                        </Show>

                        <button
                          class="w-full text-left px-3 py-2 sm:px-4 sm:py-2 hover:bg-surface-2 rounded-sm flex items-center gap-2 text-xs sm:text-sm transition-colors duration-90"
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
          <aside class="relative z-50 w-64 bg-surface/95 h-full shadow-xl flex flex-col transition-transform">
            <div class="p-4 border-b border-line flex items-center justify-between">
              <span class="font-mono font-bold text-lg text-ink">
                {t("top_bar.menu.title")}
              </span>
              <button
                onClick={() => setSidebarOpen(false)}
                class="text-ink-muted hover:text-ink"
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
                        class={`block px-4 py-2 font-mono text-sm rounded-sm transition-colors ${
                          isActive(link.href)
                            ? "text-ink bg-surface-2 border-l-2 border-accent"
                            : "text-ink-muted hover:text-ink hover:bg-surface-2"
                        }`}
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
