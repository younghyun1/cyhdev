import { spawn, type ChildProcess } from "node:child_process";
import { isAbsolute } from "node:path";
import { mkdirSync, openSync, closeSync } from "node:fs";
import { setTimeout as delay } from "node:timers/promises";
import { createServer } from "node:net";
import type { APIRequestContext } from "@playwright/test";
import { required, type Campaign } from "./model";
import { validateEnvironment, validateFixturePaths } from "./isolation";

/** Hooks manage only disposable fixtures; argv is never evaluated by a shell. */
export async function hook(argv: string[], cwd: string): Promise<void> {
  if (!argv[0]) throw new Error("Missing hook executable");
  await new Promise<void>((resolve, reject) => {
    const child = spawn(argv[0]!, argv.slice(1), { cwd, stdio: "ignore", shell: false, timeout: 120000, killSignal: "SIGKILL" });
    child.on("error", () => reject(new Error("Fixture hook failed to start")));
    child.on("exit", (code) => code === 0 ? resolve() : reject(new Error("Fixture hook failed")));
  });
}
export async function start(campaign: Campaign, request: APIRequestContext): Promise<ChildProcess> {
  const runtime = required("CYHDEV_OPT_RUNTIME");
  const binary = required("CYHDEV_BINARY");
  if (!isAbsolute(runtime) || !isAbsolute(binary)) throw new Error("Runtime and binary paths must be absolute");
  validateEnvironment(process.env, runtime);
  for (const port of [campaign.port, campaign.redirect_port]) await new Promise<void>((resolve, reject) => {
    const probe = createServer();
    probe.once("error", () => reject(new Error("Campaign port already serves a process")));
    probe.listen(port, "127.0.0.1", () => probe.close((error) => error ? reject(new Error("Port check failed")) : resolve()));
  });
  await hook(campaign.reset_command, runtime);
  validateFixturePaths(process.env, runtime);
  const run = required("CYHDEV_OPT_RUN");
  mkdirSync(`${run}/runtime-logs`, { recursive: true, mode: 0o700 });
  const log = openSync(`${run}/runtime-logs/${required("CYHDEV_OPT_STAGE")}.log`, "a", 0o600);
  const child = spawn(binary, [], {
    cwd: runtime, stdio: ["ignore", log, log], shell: false,
    env: { ...process.env, IS_AWS_ECS: "true", HOST_IP: "127.0.0.1", HOST_PORT: String(campaign.port),
      CURR_ENV: "local", HTTP_REDIRECT_PORT: String(campaign.redirect_port),
      MINECRAFT_MANAGEMENT_PORT: String(campaign.minecraft_management_port),
      MINECRAFT_MANAGEMENT_SECRET: "optimizationfixture012345678901234567890",
      PUBLIC_APP_ORIGIN: campaign.base_url,
      AWS_EC2_METADATA_DISABLED: "true", MINECRAFT_SEED_WORKER: required("CYHDEV_SEED_BINARY") },
  });
  closeSync(log);
  let failed = false;
  child.on("error", () => { failed = true; });
  for (let attempt = 0; attempt < 120; attempt += 1) {
    if (failed || child.exitCode !== null) throw new Error("Disposable backend failed to start; inspect private runtime log");
    const response = await request.get(`${campaign.base_url}/api/healthcheck/server`, { timeout: 1000 }).catch(() => null);
    if (response) {
      const ready = response.ok();
      await response.dispose();
      if (ready) return child;
    }
    await delay(500);
  }
  await stop(child);
  throw new Error("Disposable backend readiness deadline expired");
}
export async function stop(child: ChildProcess): Promise<void> {
  if (child.exitCode !== null || child.signalCode !== null) throw new Error("Disposable backend exited before campaign completion");
  const exited = new Promise<number | null>((resolve) => child.once("exit", resolve));
  child.kill("SIGTERM");
  const result = await Promise.race([exited.then((code) => ({ code })), delay(45000).then(() => null)]);
  if (result === null) { child.kill("SIGKILL"); await exited; throw new Error("Backend did not exit gracefully; profiles are invalid"); }
  if (result.code !== 0) throw new Error("Backend exit was unsuccessful; profiles are invalid");
}
