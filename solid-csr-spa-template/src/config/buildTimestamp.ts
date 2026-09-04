/** Resolves reproducible build metadata with a development-time fallback. */
export function resolveBuildTimestamp(
  sourceDateEpoch: string | undefined,
  currentTime: Date = new Date(),
  source = "SOURCE_DATE_EPOCH",
): string {
  if (sourceDateEpoch === undefined || sourceDateEpoch.length === 0) {
    return validIsoTimestamp(currentTime, "current build time");
  }
  if (!/^\d+$/.test(sourceDateEpoch)) {
    throw new Error(`${source} must be an unsigned integer`);
  }

  const epochSeconds = Number(sourceDateEpoch);
  const milliseconds = epochSeconds * 1_000;
  if (!Number.isSafeInteger(epochSeconds) || !Number.isSafeInteger(milliseconds)) {
    throw new Error(`${source} is outside JavaScript's safe integer range`);
  }
  return validIsoTimestamp(new Date(milliseconds), source);
}

/** Prefers the cache-neutral application epoch while retaining the standard override. */
export function resolveConfiguredBuildTimestamp(
  appBuildEpoch: string | undefined,
  sourceDateEpoch: string | undefined,
  currentTime: Date = new Date(),
): string {
  if (appBuildEpoch !== undefined) {
    return resolveBuildTimestamp(appBuildEpoch, currentTime, "APP_BUILD_EPOCH");
  }
  return resolveBuildTimestamp(sourceDateEpoch, currentTime);
}

function validIsoTimestamp(value: Date, source: string): string {
  if (!Number.isFinite(value.getTime())) {
    throw new Error(`${source} is outside JavaScript's supported date range`);
  }
  return value.toISOString();
}
