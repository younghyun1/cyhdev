import { Suspense, type Component, onMount, createEffect } from "solid-js";

import { useLocation } from "@solidjs/router";

import TopBar from "./components/TopBar";
import BottomBar from "./components/BottomBar";

import { theme, applyTheme } from "./state/theme"; // <-- import theme for dynamic color

import { authApi } from "./services/all_api";

import { setAuthenticated, setSuperuser, setUser } from "./state/auth";
import { setServerBuildInfo } from "./state/server_info";

const App: Component = (props: { children: Element }) => {
  const location = useLocation();

  onMount(async () => {
    try {
      const resp = await authApi.me();
      if (resp?.success && resp.data) {
        const hasUser = !!resp.data.user_info?.user_id;
        setAuthenticated(hasUser);
        setUser(hasUser ? resp.data : null);

        // Save backend build info from response
        setServerBuildInfo({
          built_time: resp.data.build_time,
          name: resp.data.axum_version,
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
        setServerBuildInfo({});
        setSuperuser(false);
      }
    } catch (e) {
      setAuthenticated(false);
      setUser(null);
      setServerBuildInfo({});
      setSuperuser(false);
    }
  });
  createEffect(() => {
    applyTheme(theme());
  });

  return (
    <div
      id="app-root"
      class="transition-colors duration-90 min-h-screen flex flex-col bg-white text-gray-900 dark:bg-gray-950 dark:text-gray-100 overflow-x-hidden"
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
