import { createResource, Show } from "solid-js";
import HostStatsDashboard from "../components/HostStatsDashboard";
import { healthApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import { theme } from "../state/theme";

export default function BackendStats() {
  const [fastfetch] = createResource(async () => {
    try {
      const res = await healthApi.fastfetch();
      return res.data;
    } catch (e) {
      console.error(e);
      return null;
    }
  });
  const isDark = () => theme() === "dark";

  return (
    <main class={pageStyles.page}>
      <div
        class={`${pageStyles.pageInner} flex flex-col xl:flex-row items-center xl:items-stretch justify-center gap-8`}
      >
        <HostStatsDashboard />

        <Show when={fastfetch()}>
          <div
            class="w-full max-w-7xl xl:w-[42rem] 2xl:w-[48rem] p-6 rounded-xl shadow-lg font-mono text-xs sm:text-sm overflow-x-auto overflow-y-auto border-2 flex flex-col"
            style={{
              background: isDark()
                ? "linear-gradient(135deg, #1f2937 0%, #111827 100%)"
                : "linear-gradient(135deg, #ffffff 0%, #f8fafc 100%)",
              "border-color": isDark() ? "#f59e0b" : "#b45309",
              color: isDark() ? "#e2e8f0" : "#0f172a",
            }}
          >
            <pre class="m-auto" innerHTML={fastfetch()} />
          </div>
        </Show>
      </div>
    </main>
  );
}
