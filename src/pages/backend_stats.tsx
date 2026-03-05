import { createResource, Show } from "solid-js";
import HostStatsDashboard from "../components/HostStatsDashboard";
import { healthApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import { theme } from "../state/theme";
import { refreshHealthState } from "../state/health";

export default function BackendStats() {
  const [fastfetch, { refetch: refetchFastfetch }] = createResource(
    async () => {
      try {
        const res = await healthApi.fastfetch();
        return res.data;
      } catch (e) {
        console.error(e);
        return null;
      }
    },
  );

  const refreshAll = async () => {
    await Promise.all([refreshHealthState(), refetchFastfetch()]);
  };

  const isDark = () => theme() === "dark";

  return (
    <main class={pageStyles.page}>
      <div
        class={`${pageStyles.pageInner} flex flex-col xl:flex-row items-center xl:items-stretch justify-center gap-8`}
      >
        <HostStatsDashboard onRefresh={refreshAll} />

        <Show when={fastfetch()}>
          <div
            class="w-full max-w-7xl xl:w-2xl 2xl:w-3xl p-6 rounded-xl shadow-lg font-mono text-xs sm:text-sm overflow-x-auto overflow-y-auto border-2 flex flex-col"
            style={{
              background: isDark()
                ? "linear-gradient(135deg, #1f2937 0%, #111827 100%)"
                : "linear-gradient(135deg, #ffffff 0%, #f8fafc 100%)",
              "border-color": isDark() ? "#f59e0b" : "#b45309",
              color: isDark() ? "#e2e8f0" : "#0f172a",
            }}
          >
            <div class="flex items-center justify-end pb-3 mb-3 border-b border-opacity-20 border-current">
              <button
                class="px-3 py-1 text-xs font-semibold rounded hover:opacity-80 transition-opacity"
                style={{
                  background: isDark() ? "#111827" : "#f3f4f6",
                  color: isDark() ? "#e2e8f0" : "#334155",
                  border: `1px solid ${isDark() ? "#374151" : "#d1d5db"}`,
                }}
                onClick={refreshAll}
              >
                Refresh
              </button>
            </div>
            <div class="m-auto whitespace-pre" innerHTML={fastfetch() ?? ""} />
          </div>
        </Show>
      </div>
    </main>
  );
}
