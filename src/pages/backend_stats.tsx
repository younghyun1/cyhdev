import { createResource, Show } from "solid-js";
import HostStatsDashboard from "../components/HostStatsDashboard";
import { healthApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";

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

  return (
    <main class={pageStyles.page}>
      <div
        class={`${pageStyles.pageInner} flex flex-col xl:flex-row items-center xl:items-stretch justify-center gap-8`}
      >
        <HostStatsDashboard />

        <Show when={fastfetch()}>
          <div
            class="w-full max-w-7xl xl:w-auto p-6 rounded-xl shadow-lg font-mono text-xs sm:text-sm overflow-x-auto overflow-y-auto border border-slate-700/70 flex flex-col"
            style={{
              background: "linear-gradient(135deg, #1f2937 0%, #111827 100%)",
              color: "#e2e8f0",
            }}
          >
            <pre class="m-auto" innerHTML={fastfetch()} />
          </div>
        </Show>
      </div>
    </main>
  );
}
