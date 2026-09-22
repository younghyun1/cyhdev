import { Loading, Show, createMemo } from "solid-js";
import DOMPurify from "dompurify";
import HostStatsDashboard from "../components/HostStatsDashboard";
import { healthApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";

export default function BackendStats() {
  const fastfetch = createMemo(async () => {
    try {
      const res = await healthApi.fastfetch();
      return res.data;
    } catch (e) {
      console.error(e);
      return null;
    }
  });

  // fastfetch HTML is server-generated ANSI->HTML (color spans); sanitize to strip any script/handlers.
  const cleanFastfetch = () =>
    DOMPurify.sanitize(fastfetch() ?? "", {
      ALLOWED_TAGS: ["span", "b", "i", "br", "pre"],
      ALLOWED_ATTR: ["style", "class"],
    });

  return (
    <main class={`${pageStyles.page} backend-stats-page`}>
      <div
        class={`${pageStyles.pageInner} backend-stats-layout max-w-[1700px] flex flex-col xl:flex-row items-center xl:items-stretch justify-center gap-8`}
      >
        <div class="w-full max-w-7xl xl:w-[56rem] 2xl:w-[64rem]">
          <HostStatsDashboard />
        </div>

        <Loading>
          <Show when={fastfetch()}>
            <div
              class="stats-panel stats-host-details max-w-7xl"
            >
              {/* eslint-disable-next-line solid/no-innerhtml -- value is DOMPurify-sanitized in cleanFastfetch(). */}
              <pre class="m-auto whitespace-pre" innerHTML={cleanFastfetch()} />
            </div>
          </Show>
        </Loading>
      </div>
    </main>
  );
}
