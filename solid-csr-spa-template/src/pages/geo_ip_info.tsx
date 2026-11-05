import { createSignal, createMemo, Loading, Show } from "solid-js";
import { geoIpApi, type IpInfo } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import { t, tx } from "../state/i18n";

export default function GeoIpInfo() {
  const [ipInput, setIpInput] = createSignal("");
  const [ipInfo, setIpInfo] = createSignal<IpInfo | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const myIpInfo = createMemo(async () => {
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
        class={`text-xl font-semibold mb-6 text-ink border-b border-line pb-2`}
      >
        {props.title}
      </h2>

      <div class="grid grid-cols-2 gap-y-4 gap-x-6">
        <div>
          <span class="block text-xs font-medium text-ink-muted uppercase tracking-wider">
            {t("geo.ip_address")}
          </span>
          <span dir="ltr" class="inline-block text-lg text-ink font-mono font-medium">
            {props.info.ip}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-ink-muted uppercase tracking-wider">
            {t("common.country")}
          </span>
          <span class="text-lg text-ink font-medium">
            {props.info.country_name}{" "}
            <span class="text-sm text-ink-muted">
              ({props.info.country_code})
            </span>
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-ink-muted uppercase tracking-wider">
            {t("geo.region_state")}
          </span>
          <span class="text-lg text-ink font-medium">
            {props.info.state || t("common.n_a")}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-ink-muted uppercase tracking-wider">
            {t("geo.city")}
          </span>
          <span class="text-lg text-ink font-medium">
            {props.info.city || t("common.n_a")}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-ink-muted uppercase tracking-wider">
            {t("geo.postal_code")}
          </span>
          <span class="text-lg text-ink font-medium">
            {props.info.postal || t("common.n_a")}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-ink-muted uppercase tracking-wider">
            {t("geo.coordinates")}
          </span>
          <div class="flex items-center gap-2">
            <span dir="ltr" class="text-ink font-mono bg-surface-2 px-2 py-1 rounded-sm text-sm">
              {props.info.latitude}, {props.info.longitude}
            </span>
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <main class={pageStyles.page}>
      <div class={pageStyles.pageInner}>
        <h1 class={`${pageStyles.title} mb-6`}>
          {t("geo.title")}
        </h1>
        <div class="geo-ip-columns">

        {/* Current Client IP Section */}
        <section>
          <h2 class={`${pageStyles.sectionTitle} mb-3`}>
            {t("geo.your_ip_info")}
          </h2>
          <Loading
            fallback={
              <div class={`${pageStyles.cardPadded} animate-pulse`}>
                <div class="h-6 bg-surface-2 rounded-sm w-1/3 mb-4" />
                <div class="grid grid-cols-2 gap-4">
                  <div class="h-4 bg-surface-2 rounded-sm" />
                  <div class="h-4 bg-surface-2 rounded-sm" />
                  <div class="h-4 bg-surface-2 rounded-sm" />
                  <div class="h-4 bg-surface-2 rounded-sm" />
                </div>
              </div>
            }
          >
            <Show
              when={myIpInfo()}
              fallback={
                <div
                  class={`${pageStyles.cardPadded} text-center text-ink-muted`}
                >
                  {t("geo.could_not_determine")}
                </div>
              }
            >
              <IpInfoDisplay info={myIpInfo()!} title={t("geo.your_connection")} />
            </Show>
          </Loading>
        </section>

        {/* IP Lookup Section */}
        <section>
          <h2 class={`${pageStyles.sectionTitle} mb-3`}>
            {t("geo.lookup_any")}
          </h2>
          <form onSubmit={handleLookup} class="mb-6">
            <div class="flex flex-col sm:flex-row gap-3">
              <input
                type="text"
                dir="ltr"
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
              // The isolate keeps an IPv6 address in order inside a right-to-left heading.
              title={tx("geo.results_for", { ip: `\u2066${ipInfo()!.ip}\u2069` })}
            />
          </Show>
        </section>
        </div>

        <p class="mt-8 text-xs text-ink-muted">
          {t("geo.attribution_prefix")}{" "}
          <a
            href="https://lite.ip2location.com"
            target="_blank"
            rel="noopener noreferrer"
            class="underline hover:text-ink-muted"
          >
            {t("geo.attribution_link")}
          </a>
          .
        </p>
      </div>
    </main>
  );
}
