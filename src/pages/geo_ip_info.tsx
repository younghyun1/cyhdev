import { createSignal, createResource, Show, Suspense } from "solid-js";
import { geoIpApi, type IpInfo } from "../services/all_api";

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
    <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg shadow-sm p-6 sm:p-8">
      <h2 class="text-xl font-bold mb-6 text-gray-800 dark:text-gray-200 border-b border-gray-100 dark:border-gray-800 pb-2">
        {props.title}
      </h2>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-y-4 gap-x-8">
        <div>
          <span class="block text-xs font-medium text-gray-500 uppercase tracking-wider">
            IP Address
          </span>
          <span class="text-lg text-gray-900 dark:text-gray-100 font-mono font-medium">
            {props.info.ip}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-gray-500 uppercase tracking-wider">
            Country
          </span>
          <span class="text-lg text-gray-900 dark:text-gray-100 font-medium">
            {props.info.country_name}{" "}
            <span class="text-sm text-gray-500">
              ({props.info.country_code})
            </span>
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-gray-500 uppercase tracking-wider">
            Region / State
          </span>
          <span class="text-lg text-gray-900 dark:text-gray-100 font-medium">
            {props.info.state || "N/A"}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-gray-500 uppercase tracking-wider">
            City
          </span>
          <span class="text-lg text-gray-900 dark:text-gray-100 font-medium">
            {props.info.city || "N/A"}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-gray-500 uppercase tracking-wider">
            Postal Code
          </span>
          <span class="text-lg text-gray-900 dark:text-gray-100 font-medium">
            {props.info.postal || "N/A"}
          </span>
        </div>

        <div>
          <span class="block text-xs font-medium text-gray-500 uppercase tracking-wider">
            Coordinates
          </span>
          <div class="flex items-center gap-2">
            <span class="text-gray-900 dark:text-gray-100 font-mono bg-gray-50 dark:bg-gray-800 px-2 py-1 rounded text-sm">
              {props.info.latitude}, {props.info.longitude}
            </span>
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <main class="max-w-3xl mx-auto py-12 px-6">
      <h1 class="text-3xl font-bold mb-8 text-gray-900 dark:text-gray-100 text-center">
        Geo-IP Database Lookup
      </h1>

      {/* Current Client IP Section */}
      <section class="mb-10">
        <h2 class="text-lg font-semibold mb-4 text-gray-800 dark:text-gray-200">
          Your IP Information
        </h2>
        <Suspense
          fallback={
            <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg shadow-sm p-6 animate-pulse">
              <div class="h-6 bg-gray-200 dark:bg-gray-700 rounded w-1/3 mb-4"></div>
              <div class="grid grid-cols-2 gap-4">
                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded"></div>
                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded"></div>
                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded"></div>
                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded"></div>
              </div>
            </div>
          }
        >
          <Show
            when={myIpInfo()}
            fallback={
              <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg shadow-sm p-6 text-center text-gray-500 dark:text-gray-400">
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
        <h2 class="text-lg font-semibold mb-4 text-gray-800 dark:text-gray-200">
          Lookup Any IP Address
        </h2>
        <form onSubmit={handleLookup} class="mb-6">
          <div class="flex flex-col sm:flex-row gap-3">
            <input
              type="text"
              value={ipInput()}
              onInput={(e) => setIpInput(e.currentTarget.value)}
              placeholder="Enter IPv4 or IPv6 address..."
              class="flex-1 px-4 py-2 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 outline-none transition"
            />
            <button
              type="submit"
              disabled={loading() || !ipInput().trim()}
              class="px-6 py-2 bg-blue-600 text-white font-semibold rounded hover:bg-blue-700 transition disabled:opacity-60 disabled:cursor-not-allowed"
            >
              {loading() ? "Searching..." : "Lookup"}
            </button>
          </div>
          <Show when={error()}>
            <div class="mt-3 text-red-600 dark:text-red-400 text-sm text-center">
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
    </main>
  );
}
