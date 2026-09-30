import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { isAbsolute, resolve } from "node:path";
import { cpus, release, totalmem } from "node:os";
import type { BrowserContext } from "@playwright/test";
import { digest, interpolate, localUrl, readJson, required, type Campaign } from "./model";
import { requestStep } from "./steps";
import { nativeSample } from "./native-result";

const sha256 = (value: unknown) => createHash("sha256").update(JSON.stringify(value)).digest("hex");
function cpuTicks(pid: number): number {
  const stat = readFileSync(`/proc/${pid}/stat`, "utf8");
  const fields = stat.slice(stat.lastIndexOf(")") + 2).split(" ");
  const ticks = Number(fields[11]) + Number(fields[12]);
  if (!Number.isFinite(ticks)) throw new Error("Invalid disposable server CPU counters");
  return ticks;
}
async function execute(argv: string[]): Promise<void> {
  await new Promise<void>((resolvePromise, reject) => {
    const child = spawn("/usr/bin/time", argv, { stdio: "ignore", shell: false, detached: true, env: { ...process.env, LC_ALL: "C" } });
    let timedOut = false;
    const timer = setTimeout(() => {
      timedOut = true;
      if (child.pid) { try { process.kill(-child.pid, "SIGKILL"); } catch { /* The owned process group may already have exited. */ } }
    }, 120000);
    child.once("error", () => { clearTimeout(timer); reject(new Error("Native load generator failed to start")); });
    child.once("close", (code) => {
      clearTimeout(timer);
      if (code !== 0 || timedOut) reject(new Error("Native load generator failed or exceeded its deadline"));
      else resolvePromise();
    });
  });
}

/** Keep Playwright semantic checks outside the native pooled HTTPS measurement loop. */
export async function nativeBenchmark(context: BrowserContext, campaign: Campaign, serverPid?: number): Promise<void> {
  const config = campaign.benchmark;
  if (!config || !serverPid) throw new Error("Native measurements require the owned backend process");
  const executable = required("CYHDEV_OHA");
  if (!isAbsolute(executable)) throw new Error("CYHDEV_OHA must name an absolute executable");
  const executableDigest = digest(executable);
  const version = execFileSync(executable, ["--version"], { encoding: "utf8", timeout: 5000 }).trim();
  const ticksPerSecond = Number(execFileSync("/usr/bin/getconf", ["CLK_TCK"], { encoding: "utf8", timeout: 5000 }).trim());
  if (!Number.isInteger(ticksPerSecond) || ticksPerSecond <= 0) throw new Error("Invalid process CPU clock rate");
  const directory = resolve(required("CYHDEV_OPT_RUN"), "native-loadgen", required("CYHDEV_OPT_STAGE"));
  mkdirSync(directory, { recursive: true, mode: 0o700 });
  const urls = config.cases.map((step) => localUrl(campaign.base_url, interpolate(step.path, campaign.parameters)));
  const targetFile = `${directory}/urls.txt`;
  writeFileSync(targetFile, `${urls.join("\n")}\n`, { mode: 0o600 });
  for (const step of config.cases)
    if (!await requestStep(context, campaign, step, { ...campaign.parameters })) throw new Error("Native workload semantic preflight failed");
  // Release the preflight connection before using the server's bounded connection allowance.
  await context.close();
  const workerThreads = config.worker_threads ?? 4;
  const environment = { declared: config.environment, cpu: cpus().map(({ model }) => ({ model })), kernel: release(), memory: totalmem(),
    topology: "native-oha-https-http1.1-pooled-loopback", executable_sha256: executableDigest, version, worker_threads: workerThreads,
    concurrency: config.concurrency, requests_per_sample: config.requests_per_sample, warmup_requests: config.warmup_requests,
    client_cpu_set: readFileSync("/proc/self/status", "utf8").match(/^Cpus_allowed_list:\s*(.+)$/m)?.[1],
    server_cpu_set: process.env.CYHDEV_OPT_SERVER_CPU_SET ?? null, server_worker_threads: process.env.TOKIO_WORKER_THREADS ?? null,
    encoding: "identity", timeout_seconds: 2, base_url: campaign.base_url };
  async function measure(label: string, requests: number) {
    const report = `${directory}/${label}.json`;
    const cpu = `${directory}/${label}-cpu.json`;
    const before = cpuTicks(serverPid!);
    await execute(["-f", '{"user_seconds":%U,"system_seconds":%S,"elapsed_seconds":%e,"max_rss_kib":%M}', "-o", cpu,
      executable, "--no-tui", "--output-format", "json", "--output", report, "--http-version", "1.1", "--insecure",
      "--worker-threads", String(workerThreads), "-n", String(requests), "-c", String(config!.concurrency), "-t", "2s",
      "-H", `Origin: ${new URL(campaign.base_url).origin}`, "-H", "Accept: application/json", "-H", "Accept-Encoding: identity",
      "--urls-from-file", targetFile]);
    const raw = readJson<unknown>(report);
    const sample = nativeSample(raw, requests);
    const usage = readJson<Record<string, number>>(cpu);
    usage.server_cpu_seconds = (cpuTicks(serverPid!) - before) / ticksPerSecond;
    writeFileSync(cpu, JSON.stringify(usage, null, 2), { mode: 0o600 });
    return sample;
  }
  await measure("warmup", config.warmup_requests);
  const samples = [];
  for (let sample = 0; sample < config.samples; sample += 1) samples.push(await measure(`sample-${sample + 1}`, config.requests_per_sample));
  if (digest(executable) !== executableDigest) throw new Error("Native load generator changed during measurement");
  writeFileSync(`${directory}/environment.json`, JSON.stringify(environment, null, 2), { mode: 0o600 });
  writeFileSync(required("CYHDEV_BENCHMARK_REPORT"), JSON.stringify({ schema_version: 1, stage: required("CYHDEV_OPT_STAGE"),
    binary_sha256: required("CYHDEV_BINARY_SHA256"), environment_sha256: sha256(environment),
    workload_sha256: sha256({ cases: config.cases, campaign: required("CYHDEV_CAMPAIGN_SHA256") }), samples }, null, 2), { mode: 0o600 });
}
