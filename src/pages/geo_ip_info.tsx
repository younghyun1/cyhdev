import { createSignal, createResource, Show, Suspense } from "solid-js";
import { geoIpApi, type IpInfo } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";

export default function GeoIpInfo() {
  const [ipInput, setIpInput] = createSignal("");
  const [ipInfo, setIpInfo] = createSignal<IpInfo | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const [myIpInfo] = createResource(async () => {
    try {
      const response = await geoIpApi.getMyIpInfo();
      return response.data;
    } catch {
      return null;
    }
  });

  const handleLookup = async (e: Event) => {
    e.preventDefault();
    if (!ipInput().trim()) return;

    setLoading(true);
    setError(null);
    setIpInfo(null);

    try {
      const response = await geoIpApi.getGeoIpInfo(ipInput().trim());
      setIpInfo(response.data);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to lookup IP information",
      );
    } finally {
      setLoading(false);
    }
  };

  const IpInfoDisplay = (props: { info: IpInfo; title: string }) => (
    <div class={pageStyles.cardPadded}>
      <h2
        class={`text-xl font-semibold mb-6 text-slate-800 dark:text-slate-200 border-b border-slate-100 dark:border-slate-800 pb-2`}
      >
        {props.title}
      </h2>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-y-4 gap-x-8">
        <div>
          <span class="block text-xs font-medium text-slate-500 uppercase tracking-wider">
            IP Address
          </span>
          <span class="text-lg text-slate-900 dark:text-slate-100 font-mono font-medium">
            {props.info.ip}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-slate-500 uppercase tracking-wider">
            Country
          </span>
          <span class="text-lg text-slate-900 dark:text-slate-100 font-medium">
            {props.info.country_name}{" "}
            <span class="text-sm text-slate-500">
              ({props.info.country_code})
            </span>
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-slate-500 uppercase tracking-wider">
            Region / State
          </span>
          <span class="text-lg text-slate-900 dark:text-slate-100 font-medium">
            {props.info.state || "N/A"}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-slate-500 uppercase tracking-wider">
            City
          </span>
          <span class="text-lg text-slate-900 dark:text-slate-100 font-medium">
            {props.info.city || "N/A"}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-slate-500 uppercase tracking-wider">
            Postal Code
          </span>
          <span class="text-lg text-slate-900 dark:text-slate-100 font-medium">
            {props.info.postal || "N/A"}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-slate-500 uppercase tracking-wider">
            Coordinates
          </span>
          <div class="flex items-center gap-2">
            <span class="text-slate-900 dark:text-slate-100 font-mono bg-slate-50 dark:bg-slate-800 px-2 py-1 rounded text-sm">
              {props.info.latitude}, {props.info.longitude}
            </span>
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <main class={pageStyles.page}>
      <div class={pageStyles.pageInnerNarrow}>
        <h1 class={`${pageStyles.title} mb-8 text-center`}>
          Geo-IP Database Lookup
        </h1>

        {/* Current Client IP Section */}
        <section class="mb-10">
          <h2 class={`${pageStyles.sectionTitle} mb-4`}>Your IP Information</h2>
          <Suspense
            fallback={
              <div class={`${pageStyles.cardPadded} animate-pulse`}>
                <div class="h-6 bg-slate-200 dark:bg-slate-700 rounded w-1/3 mb-4"></div>
                <div class="grid grid-cols-2 gap-4">
                  <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded"></div>
                  <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded"></div>
                  <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded"></div>
                  <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded"></div>
                </div>
              </div>
            }
          >
            <Show
              when={myIpInfo()}
              fallback={
                <div
                  class={`${pageStyles.cardPadded} text-center text-slate-500 dark:text-slate-400`}
                >
                  Could not determine your IP information.
                </div>
              }
            >
              <IpInfoDisplay info={myIpInfo()!} title="Your Connection" />
            </Show>
          </Suspense>
        </section>

        {/* IP Lookup Section */}
        <section>
          <h2 class={`${pageStyles.sectionTitle} mb-4`}>
            Lookup Any IP Address
          </h2>
          <form onSubmit={handleLookup} class="mb-6">
            <div class="flex flex-col sm:flex-row gap-3">
              <input
                type="text"
                value={ipInput()}
                onInput={(e) => setIpInput(e.currentTarget.value)}
                placeholder="Enter IPv4 or IPv6 address..."
                class={`${pageStyles.input} flex-1`}
              />
              <button
                type="submit"
                disabled={loading() || !ipInput().trim()}
                class={`${pageStyles.buttonPrimary} px-6 py-2 disabled:opacity-60 disabled:cursor-not-allowed`}
              >
                {loading() ? "Searching..." : "Lookup"}
              </button>
            </div>
            <Show when={error()}>
              <div class={`${pageStyles.alertError} mt-3 text-center`}>
                {error()}
              </div>
            </Show>
          </form>

          <Show when={ipInfo()}>
            <IpInfoDisplay
              info={ipInfo()!}
              title={`Results for ${ipInfo()!.ip}`}
            />
          </Show>
        </section>

        <p class="mt-10 text-center text-xs text-slate-400 dark:text-slate-500">
          This site uses the IP2Location LITE database for{" "}
          <a
            href="https://lite.ip2location.com"
            target="_blank"
            rel="noopener noreferrer"
            class="underline hover:text-slate-600 dark:hover:text-slate-300"
          >
            IP geolocation
          </a>
          .
        </p>
      </div>
    </main>
  );
}
