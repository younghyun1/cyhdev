import { cpus, release, totalmem } from "node:os";
import { createHash } from "node:crypto";
import { writeFileSync } from "node:fs";
import type { Browser } from "@playwright/test";
import { requestStep } from "./steps";
import { required, type Campaign } from "./model";
import { actorContext } from "./sessions";

/** Measure the same explicitly weighted workload with TLS and reused connections. */
export async function benchmark(browser: Browser, campaign: Campaign): Promise<void> {
  const config = campaign.benchmark;
  if (!config) throw new Error("Missing repeated benchmark configuration");
  const context = await actorContext(browser, campaign, config.actor);
  const variables = { ...campaign.parameters };
  async function replay(count: number): Promise<number[]> {
    let cursor = 0;
    const durations: number[] = [];
    await Promise.all(Array.from({ length: config!.concurrency }, async () => {
      while (cursor < count) {
        const index = cursor++;
        const step = config!.cases[index % config!.cases.length];
        if (!step) throw new Error("Empty benchmark workload");
        const start = performance.now();
        if (!await requestStep(context, campaign, step, { ...variables })) throw new Error("Benchmarks require successful requests");
        durations.push(performance.now() - start);
      }
    }));
    return durations;
  }
  try {
    await replay(config.warmup_requests);
    const samples = [];
    for (let sample = 0; sample < config.samples; sample += 1) {
      const start = performance.now();
      const durations = await replay(config.requests_per_sample);
      const elapsed = performance.now() - start;
      durations.sort((a, b) => a - b);
      samples.push({ requests_per_second: durations.length * 1000 / elapsed,
        p95_ms: durations[Math.ceil(durations.length * 0.95) - 1], p99_ms: durations[Math.ceil(durations.length * 0.99) - 1], failures: 0 });
    }
    const environment = { declared: config.environment, cpu: cpus().map(({ model }) => ({ model })),
      kernel: release(), memory: totalmem(), node: process.version, concurrency: config.concurrency,
      topology: "playwright-https-loopback-reused-connections", base_url: campaign.base_url };
    const sha256 = (value: unknown) => createHash("sha256").update(JSON.stringify(value)).digest("hex");
    writeFileSync(required("CYHDEV_BENCHMARK_REPORT"), JSON.stringify({ schema_version: 1, stage: required("CYHDEV_OPT_STAGE"),
      binary_sha256: required("CYHDEV_BINARY_SHA256"), environment_sha256: sha256(environment),
      workload_sha256: sha256({ cases: config.cases, campaign: required("CYHDEV_CAMPAIGN_SHA256") }), samples }, null, 2), { mode: 0o600 });
  } finally { await context.close(); }
}
