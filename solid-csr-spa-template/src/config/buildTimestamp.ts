/** Resolves reproducible build metadata with a development-time fallback. */
export function resolveBuildTimestamp(
  sourceDateEpoch: string | undefined,
  currentTime: Date = new Date(),
): string {
  if (sourceDateEpoch === undefined || sourceDateEpoch.length === 0) {
    return validIsoTimestamp(currentTime, "current build time");
  }
  if (!/^\d+$/.test(sourceDateEpoch)) {
    throw new Error("SOURCE_DATE_EPOCH must be an unsigned integer");
  }

  const epochSeconds = Number(sourceDateEpoch);
  const milliseconds = epochSeconds * 1_000;
  if (!Number.isSafeInteger(epochSeconds) || !Number.isSafeInteger(milliseconds)) {
    throw new Error("SOURCE_DATE_EPOCH is outside JavaScript's safe integer range");
  }
  return validIsoTimestamp(new Date(milliseconds), "SOURCE_DATE_EPOCH");
}

function validIsoTimestamp(value: Date, source: string): string {
  if (!Number.isFinite(value.getTime())) {
    throw new Error(`${source} is outside JavaScript's supported date range`);
  }
  return value.toISOString();
}
