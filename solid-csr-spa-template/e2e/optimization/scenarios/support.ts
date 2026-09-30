import { expect, type BrowserContext, type Page } from "@playwright/test";
import { setTimeout as delay } from "node:timers/promises";
import { pointer, required, type Campaign, type Json, type RequestStep } from "../model";
import { requestStep } from "../steps";

export interface Scene {
  context: BrowserContext; page: Page; campaign: Campaign;
  variables: Record<string, string>; operations: Set<string>;
}
export const PASSWORD = "OptimizationFixture123";
export const FIXTURE = "http://127.0.0.1:34901";
export function value(json: unknown, path: string): string {
  const result = pointer(json, path);
  if (typeof result !== "string" && typeof result !== "number") throw new Error(`Expected scalar at ${path}`);
  return String(result);
}
export async function call(scene: Scene, method: string, route: string, body?: Json, options: Partial<RequestStep> = {}): Promise<unknown> {
  let result: unknown;
  const step: RequestStep = { kind: "request", method, route, path: route, status: 200, json_pointer: "/success", equals: true,
    ...(body === undefined ? {} : { body }), ...options };
  if (await requestStep(scene.context, scene.campaign, step, scene.variables, (json) => { result = json; })) scene.operations.add(`${method} ${route}`);
  return result;
}
export async function login(scene: Scene, email: string, password = PASSWORD): Promise<void> {
  await call(scene, "POST", "/api/auth/login", { user_email: email, user_password: password });
}
export async function mailToken(recipient: string,kind="verify-email"): Promise<string> {
  for (let attempt = 0; attempt < 30; attempt += 1) {
    const response = await fetch(`${FIXTURE}/__fixture/mail?recipient=${encodeURIComponent(recipient)}`, { signal: AbortSignal.timeout(5000) });
    if (response.ok) {
      const json: unknown = await response.json();
      const message = value(json, "/message").replace(/=\r?\n/g, "").replace(/=([A-Fa-f0-9]{2})/g, (_match: string, hex: string) => String.fromCharCode(parseInt(hex, 16)));
      const token = /(?:#token=|#email_verification_token=|#password_reset_token=)([A-Za-z0-9_-]{43})/.exec(message)?.[1];
      if (token && message.includes(`/${kind}`)) return token;
    }
    await delay(200);
  }
  throw new Error("Fixture mail did not contain a capability token");
}
export async function another(scene: Scene, actor: "anonymous" | "member" | "admin", action: (scene: Scene) => Promise<void>): Promise<void> {
  const browser = scene.context.browser();
  if (!browser) throw new Error("Browser is unavailable");
  const { actorContext } = await import("../sessions");
  const context = await actorContext(browser, scene.campaign, actor);
  try { await action({ ...scene, context, page: await context.newPage() }); } finally { await context.unrouteAll({behavior:"ignoreErrors"}); await context.close(); }
}
export async function visible(scene: Scene, path: string, selector: string): Promise<void> {
  await scene.page.goto(new URL(path, scene.campaign.base_url).href);
  await expect(scene.page.locator(selector)).toBeVisible();
}
export function uploadFile(name: string, type: string): { file: string; mime_type: string } {
  return { file: `${required("CYHDEV_OPT_RUNTIME")}/${name}`, mime_type: type };
}
