import { createSignal, createResource, Show, Suspense } from "solid-js";
import { geoIpApi, type IpInfo } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import { t, tx } from "../state/i18n";

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
        err instanceof Error ? err.message : t("geo.lookup_failed"),
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
            {t("geo.ip_address")}
          </span>
          <span class="text-lg text-slate-900 dark:text-slate-100 font-mono font-medium">
            {props.info.ip}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-slate-500 uppercase tracking-wider">
            {t("common.country")}
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
            {t("geo.region_state")}
          </span>
          <span class="text-lg text-slate-900 dark:text-slate-100 font-medium">
            {props.info.state || t("common.n_a")}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-slate-500 uppercase tracking-wider">
            {t("geo.city")}
          </span>
          <span class="text-lg text-slate-900 dark:text-slate-100 font-medium">
            {props.info.city || t("common.n_a")}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-slate-500 uppercase tracking-wider">
            {t("geo.postal_code")}
          </span>
          <span class="text-lg text-slate-900 dark:text-slate-100 font-medium">
            {props.info.postal || t("common.n_a")}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-slate-500 uppercase tracking-wider">
            {t("geo.coordinates")}
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
          {t("geo.title")}
        </h1>

        {/* Current Client IP Section */}
        <section class="mb-10">
          <h2 class={`${pageStyles.sectionTitle} mb-4`}>
            {t("geo.your_ip_info")}
          </h2>
          <Suspense
            fallback={
              <div class={`${pageStyles.cardPadded} animate-pulse`}>
                <div class="h-6 bg-slate-200 dark:bg-slate-700 rounded w-1/3 mb-4" />
                <div class="grid grid-cols-2 gap-4">
                  <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded" />
                  <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded" />
                  <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded" />
                  <div class="h-4 bg-slate-200 dark:bg-slate-700 rounded" />
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
                  {t("geo.could_not_determine")}
                </div>
              }
            >
              <IpInfoDisplay info={myIpInfo()!} title={t("geo.your_connection")} />
            </Show>
          </Suspense>
        </section>

        {/* IP Lookup Section */}
        <section>
          <h2 class={`${pageStyles.sectionTitle} mb-4`}>
            {t("geo.lookup_any")}
          </h2>
          <form onSubmit={handleLookup} class="mb-6">
            <div class="flex flex-col sm:flex-row gap-3">
              <input
                type="text"
                value={ipInput()}
                onInput={(e) => setIpInput(e.currentTarget.value)}
                placeholder={t("geo.input_placeholder")}
                class={`${pageStyles.input} flex-1`}
              />
              <button
                type="submit"
                disabled={loading() || !ipInput().trim()}
                class={`${pageStyles.buttonPrimary} px-6 py-2 disabled:opacity-60 disabled:cursor-not-allowed`}
              >
                {loading() ? t("common.searching") : t("common.lookup")}
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
              title={tx("geo.results_for", { ip: ipInfo()!.ip })}
            />
          </Show>
        </section>

        <p class="mt-10 text-center text-xs text-slate-400 dark:text-slate-500">
          {t("geo.attribution_prefix")}{" "}
          <a
            href="https://lite.ip2location.com"
            target="_blank"
            rel="noopener noreferrer"
            class="underline hover:text-slate-600 dark:hover:text-slate-300"
          >
            {t("geo.attribution_link")}
          </a>
          .
        </p>
      </div>
    </main>
  );
}
