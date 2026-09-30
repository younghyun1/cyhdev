import { openSync, closeSync, readSync, fstatSync } from "node:fs";
import { createHash } from "node:crypto";

export type Actor = "anonymous" | "member" | "admin";
export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
export interface ActorConfig { login?: RequestStep }
export interface PageCase { path?: string; selector: string; text?: string; actor?: Actor }
export interface RequestStep {
  kind: "request"; method: string; route: string; path: string; status: number;
  body?: Json; headers?: Record<string, string>; multipart?: Record<string, string | { file: string; mime_type: string }>;
  json_pointer?: string; equals?: Json; capture?: Record<string, string>; minimum_bytes?: number; content_type?: string;
}
export interface BrowserStep {
  kind: "browser"; path: string; selector: string; text?: string;
  actions: ({ kind: "click"; selector: string } | { kind: "fill"; selector: string; value: string }
    | { kind: "upload"; selector: string; file: string } | { kind: "press"; selector: string; key: string })[];
}
export interface SocketStep { kind: "websocket"; path: string; send: Json[]; expect: string[] }
export interface CommandStep { kind: "command"; argv: string[] }
export interface FixtureScenarioStep { kind: "fixture_scenario"; name: string; operations: string[] }
export type Step = RequestStep | BrowserStep | SocketStep | CommandStep | FixtureScenarioStep;
export interface Workflow { name: string; actor: Actor; steps: Step[]; repetitions?: number }
export interface Benchmark {
  environment: Record<string, Json>; samples: number; warmup_requests: number;
  requests_per_sample: number; concurrency: number; actor: Actor; cases: RequestStep[];
}
export interface Campaign {
  schema_version: number; base_url: string; port: number; redirect_port: number; minecraft_management_port: number; reset_command: string[];
  parameters: Record<string, string>; actors: Record<Actor, ActorConfig>; pages: Record<string, PageCase>;
  workflows: Workflow[]; benchmark?: Benchmark;
  fixture_parameters?: string;
}

export function readJson<T>(path: string, limit = 1024 * 1024): T {
  const file = openSync(path, "r");
  try {
    if (fstatSync(file).size > limit) throw new Error(`JSON input exceeds ${limit} bytes`);
    const bytes = Buffer.alloc(limit + 1);
    let length = 0;
    while (length < bytes.length) {
      const count = readSync(file, bytes, length, bytes.length - length, null);
      if (count === 0) break;
      length += count;
    }
    if (length > limit) throw new Error(`JSON input exceeds ${limit} bytes`);
    return JSON.parse(bytes.subarray(0, length).toString("utf8")) as T;
  } finally { closeSync(file); }
}
export function digest(path: string): string {
  const file = openSync(path, "r");
  const hash = createHash("sha256");
  const bytes = Buffer.alloc(65536);
  try {
    for (;;) {
      const count = readSync(file, bytes, 0, bytes.length, null);
      if (count === 0) break;
      hash.update(bytes.subarray(0, count));
    }
    return hash.digest("hex");
  } finally { closeSync(file); }
}
export function required(name: string): string {
  const value = process.env[name];
  if (!value) throw new Error(`Missing ${name}`);
  return value;
}
export function localUrl(base: string, path: string): string {
  const target = new URL(path, base);
  if (!path.startsWith("/") || path.startsWith("//") || target.origin !== new URL(base).origin)
    throw new Error("Campaign requests must remain on the configured loopback origin");
  return target.href;
}
export function interpolate(value: string, variables: Record<string, string>): string {
  return value.replace(/\$\{([^}]+)\}/g, (_match: string, key: string) => {
    const replacement = variables[key] ?? process.env[key];
    if (replacement === undefined) throw new Error(`Missing fixture variable ${key}`);
    return replacement;
  });
}
export function pointer(value: unknown, path: string): unknown {
  if (path === "") return value;
  if (!path.startsWith("/")) throw new Error("JSON pointer must start with /");
  let current: unknown = value;
  for (const part of path.slice(1).split("/")) {
    const key = part.replace(/~1/g, "/").replace(/~0/g, "~");
    if (current === null || typeof current !== "object" || !(key in current)) throw new Error("JSON pointer did not resolve");
    current = (current as Record<string, unknown>)[key];
  }
  return current;
}
export function validate(campaign: Campaign): void {
  const url = new URL(campaign.base_url);
  if (campaign.schema_version !== 1 || url.protocol !== "https:" || url.hostname !== "127.0.0.1"
    || url.username || url.password || url.search || url.hash || url.pathname !== "/"
    || Number(url.port) !== campaign.port || campaign.port < 1024 || campaign.port > 65535)
    throw new Error("Campaign requires an explicit unprivileged HTTPS IPv4 loopback port");
  for (const port of [campaign.redirect_port, campaign.minecraft_management_port])
    if (!Number.isInteger(port) || port < 1024 || port > 65535 || port === campaign.port) throw new Error("Fixture ports must be distinct and unprivileged");
  if (campaign.redirect_port === campaign.minecraft_management_port) throw new Error("Fixture ports overlap");
  if (Object.keys(campaign.parameters).length > 1024 || Object.values(campaign.parameters).some((value) => typeof value !== "string" || value.length > 4096))
    throw new Error("Fixture variables exceed count or string bounds");
  if (!Array.isArray(campaign.reset_command) || campaign.reset_command.length === 0 || campaign.reset_command.length > 64)
    throw new Error("An explicit disposable fixture reset command is required");
  for (const actor of ["anonymous", "member", "admin"] as const) {
    if (!campaign.actors[actor] || (actor !== "anonymous" && !campaign.actors[actor].login))
      throw new Error(`Missing real session fixture for ${actor}`);
    const login = campaign.actors[actor].login;
    if (login) {
      validateStep(login);
      if (login.kind !== "request" || login.method !== "POST" || login.route !== "/api/auth/login" || login.status !== 200)
        throw new Error("Actor logins must exercise the real password login endpoint");
    }
  }
  if (!Array.isArray(campaign.workflows) || campaign.workflows.length > 256) throw new Error("Invalid workflows");
  const names = new Set<string>();
  for (const flow of campaign.workflows) {
    if (!flow.name || names.has(flow.name) || !campaign.actors[flow.actor] || !Array.isArray(flow.steps)
      || flow.steps.length === 0 || flow.steps.length > 1024 || !Number.isInteger(flow.repetitions ?? 1)
      || (flow.repetitions ?? 1) < 1 || (flow.repetitions ?? 1) > 100) throw new Error("Invalid or duplicate workflow");
    names.add(flow.name);
    for (const step of flow.steps) validateStep(step);
  }
  if (campaign.benchmark) {
    const b = campaign.benchmark;
    for (const [value, minimum, maximum] of [[b.samples, 5, 100], [b.warmup_requests, 1, 10000], [b.requests_per_sample, 100, 100000], [b.concurrency, 1, 16]])
      if (value === undefined || minimum === undefined || maximum === undefined || !Number.isInteger(value) || value < minimum || value > maximum)
        throw new Error("Invalid bounded benchmark count");
    if (b.requests_per_sample * b.samples > 1000000 || !campaign.actors[b.actor] || !b.environment || b.cases.length === 0 || b.cases.length > 1024)
      throw new Error("Invalid benchmark workload");
    for (const step of b.cases) validateStep(step);
  }
}
function validateStep(step: Step): void {
  if (step.kind === "request") {
    if (!/^(GET|HEAD|POST|PUT|PATCH|DELETE|OPTIONS)$/.test(step.method) || !Number.isInteger(step.status)
      || step.status < 200 || step.status > 499 || !step.route.startsWith("/")
      || (step.json_pointer === undefined && step.minimum_bytes === undefined))
      throw new Error("HTTP steps require a method, declared status and semantic or binary assertion");
  } else if (step.kind === "browser") {
    if (!step.selector || !Array.isArray(step.actions) || step.actions.length > 128) throw new Error("Invalid browser step");
  } else if (step.kind === "websocket") {
    if (!step.path.startsWith("/ws/") || step.send.length > 128 || step.expect.length === 0 || step.expect.length > 128)
      throw new Error("WebSocket steps require bounded messages and expected replies");
  } else if (step.kind === "command") {
    if (!Array.isArray(step.argv) || step.argv.length === 0 || step.argv.length > 64) throw new Error("Invalid command step");
  } else if (step.kind === "fixture_scenario") {
    if (!step.name || step.operations.length > 128 || step.operations.some((operation) => !/^(GET|POST|PATCH|DELETE) \/api\//.test(operation)))
      throw new Error("Invalid built-in fixture scenario");
  } else { throw new Error("Unknown workflow step"); }
}
