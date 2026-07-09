import {
  ErrorBoundary,
  Suspense,
  type ParentComponent,
  onMount,
  createEffect,
} from "solid-js";
import TopBar from "./components/TopBar";
import BottomBar from "./components/BottomBar";
import { theme, applyTheme } from "./state/theme";
import { authApi } from "./services/all_api";
import { fetchServerBuildInfo } from "./services/api";
import { setAuthenticated, setSuperuser, setUser } from "./state/auth";
import { updateServerBuildInfo } from "./state/server_info";
import { applyLocale, loadUiTextBundle, locale, t } from "./state/i18n";
import { LiveChatSocketProvider } from "./state/live_chat_socket";
import { RtcProvider } from "./state/rtc";

const App: ParentComponent = (props) => {
  onMount(async () => {
    applyLocale(locale());
    void loadUiTextBundle(locale());

    fetchServerBuildInfo()
      .then((info) => {
        updateServerBuildInfo({
          built_time: info.build_time,
          name: info.axum_version,
          rust_version: info.rust_version,
        });
      })
      .catch(() => {
        // Non-fatal; header-based updates can still populate build info.
      });
    try {
      const resp = await authApi.me();
      if (resp?.success && resp.data) {
        const hasUser = !!resp.data.user_info?.user_id;
        setAuthenticated(hasUser);
        setUser(hasUser ? resp.data : null);

        // Save backend build info from response without clobbering header-based info
        updateServerBuildInfo({
          built_time: resp.data.build_time,
          name: resp.data.axum_version,
          rust_version: resp.data.rust_version,
        });

        if (hasUser) {
          try {
            const superuserResp = await authApi.isSuperuser();
            setSuperuser(!!superuserResp.data?.is_superuser);
          } catch {
            setSuperuser(false);
          }
        } else {
          setSuperuser(false);
        }
      } else {
        setAuthenticated(false);
        setUser(null);
        setSuperuser(false);
      }
    } catch {
      setAuthenticated(false);
      setUser(null);
      setSuperuser(false);
    }
  });
  createEffect(() => {
    applyTheme(theme());
  });

  return (
    <LiveChatSocketProvider>
      <RtcProvider>
        <div
          id="app-root"
          class="transition-colors duration-90 min-h-screen flex flex-col bg-transparent text-ink overflow-x-hidden"
        >
          <TopBar />

          <main class="flex-1 min-h-0 pb-10 pt-12 sm:pt-14">
            <ErrorBoundary
              fallback={(err, reset) => (
                <div class="flex flex-col items-center justify-center min-h-[40vh] px-4 text-center">
                  <h2 class="text-2xl font-bold text-danger mb-2">
                    {t("app.error.title")}
                  </h2>
                  <p class="text-sm text-ink-muted mb-4 max-w-md">
                    {err instanceof Error ? err.message : t("app.error.unknown")}
                  </p>
                  <button
                    class="px-4 py-2 text-sm font-medium rounded-sm border border-line hover:bg-surface-2 transition-colors"
                    onClick={reset}
                  >
                    {t("app.error.try_again")}
                  </button>
                </div>
              )}
            >
              <Suspense>{props.children}</Suspense>
            </ErrorBoundary>
          </main>

          <BottomBar />
        </div>
      </RtcProvider>
    </LiveChatSocketProvider>
  );
};

export default App;
