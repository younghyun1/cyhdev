/** Validate native load results before they can enter artifact acceptance comparisons. */
export interface NativeSample { requests_per_second: number; p95_ms: number; p99_ms: number; failures: number }
function object(value: unknown): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) throw new Error("Invalid native result object");
  return value as Record<string, unknown>;
}
function positive(value: unknown): number {
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) throw new Error("Invalid native measurement");
  return value;
}
export function nativeSample(raw: unknown, expectedRequests: number): NativeSample {
  const report = object(raw);
  const summary = object(report.summary);
  const statuses = object(report.statusCodeDistribution);
  if (summary.successRate !== 1 || Object.keys(object(report.errorDistribution)).length !== 0
    || Object.keys(statuses).length !== 1 || statuses["200"] !== expectedRequests)
    throw new Error("Native benchmark requires every scheduled request to return HTTP 200 without transport errors");
  positive(summary.total);
  const metrics = object(report.metrics);
  const latency = object(metrics.latency_ms);
  const sample = { requests_per_second: positive(summary.requestsPerSec), p95_ms: positive(latency.p95), p99_ms: positive(latency.p99), failures: 0 };
  if (sample.p99_ms < sample.p95_ms) throw new Error("Invalid native percentile order");
  return sample;
}
