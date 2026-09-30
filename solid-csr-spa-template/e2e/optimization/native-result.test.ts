import { describe, expect, it } from "vitest";
import { nativeSample } from "./native-result";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { validate, type Campaign } from "./model";

const result = () => ({ summary: { successRate: 1, total: 2, requestsPerSec: 50000 },
  metrics: { latency_ms: { p95: 0.5, p99: 0.8 } }, statusCodeDistribution: { "200": 100000 }, errorDistribution: {} });
describe("native capacity evidence", () => {
  it("accepts complete successful samples with millisecond latencies", () => {
    expect(nativeSample(result(), 100000)).toEqual({ requests_per_second: 50000, p95_ms: 0.5, p99_ms: 0.8, failures: 0 });
  });
  it("rejects HTTP errors even when the transport reports success", () => {
    const raw = result(); Object.assign(raw.statusCodeDistribution, { "500": 1 });
    expect(() => nativeSample(raw, 100000)).toThrow();
  });
  it("rejects partial counts, connection errors and nonfinite or reversed metrics", () => {
    expect(() => nativeSample(result(), 100001)).toThrow();
    const errors = result(); Object.assign(errors.errorDistribution, { timeout: 1 });
    expect(() => nativeSample(errors, 100000)).toThrow();
    for (const value of [0, Number.NaN, Infinity]) {
      const raw = result(); raw.summary.requestsPerSec = value;
      expect(() => nativeSample(raw, 100000)).toThrow();
    }
    const raw = result(); raw.metrics.latency_ms.p99 = 0.1;
    expect(() => nativeSample(raw, 100000)).toThrow();
  });
  it("admits native concurrency while rejecting stateful workloads and excessive counts", () => {
    const campaign = JSON.parse(readFileSync(resolve(import.meta.dirname, "../../../tools/optimization/campaign.example.json"), "utf8")) as Campaign;
    if (!campaign.benchmark) throw new Error("Missing example benchmark");
    Object.assign(campaign.benchmark, { engine: "oha", worker_threads: 4, concurrency: 64, samples: 5, requests_per_sample: 1000000 });
    expect(() => validate(campaign)).not.toThrow();
    campaign.benchmark.cases[0]!.capture = { entity: "/data/id" };
    expect(() => validate(campaign)).toThrow();
    delete campaign.benchmark.cases[0]!.capture;
    campaign.benchmark.concurrency = 65;
    expect(() => validate(campaign)).toThrow();
    campaign.benchmark.concurrency = 32; campaign.benchmark.worker_threads = 17;
    expect(() => validate(campaign)).toThrow();
  });
});
