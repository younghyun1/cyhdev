import {
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

const App: ParentComponent = (props) => {
  onMount(async () => {
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
    <div
      id="app-root"
      class="transition-colors duration-90 min-h-screen flex flex-col bg-transparent text-slate-900 dark:text-slate-100 overflow-x-hidden"
    >
      <TopBar />

      <main class="flex-1 min-h-0 pb-10 pt-12 sm:pt-14">
        <Suspense>{props.children}</Suspense>
      </main>

      <BottomBar />
    </div>
  );
};

export default App;
