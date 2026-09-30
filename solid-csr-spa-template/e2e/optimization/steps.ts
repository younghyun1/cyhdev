import { readFileSync, statSync } from "node:fs";
import { expect, type BrowserContext, type Page } from "@playwright/test";
import { isDeepStrictEqual } from "node:util";
import { hook } from "./lifecycle";
import { interpolate, localUrl, pointer, required, type Campaign, type Json, type RequestStep, type Step } from "./model";

const RESPONSE_LIMIT = 16 * 1024 * 1024;
export function operationMatches(template: string, path: string): boolean {
  const expected = template.split("/");
  const actual = new URL(path, "https://127.0.0.1").pathname.split("/");
  return expected.length === actual.length && expected.every((part, index) =>
    /^\{[^}]+\}$/.test(part) ? Boolean(actual[index]) : part === actual[index]);
}
function expandJson(value: Json, variables: Record<string, string>): Json {
  if (typeof value === "string") return interpolate(value, variables);
  if (Array.isArray(value)) return value.map((item) => expandJson(item, variables));
  if (value && typeof value === "object") return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, expandJson(item, variables)]));
  return value;
}
/** Dispose each response immediately so long campaigns do not retain response buffers. */
export async function requestStep(context: BrowserContext, campaign: Campaign, step: RequestStep, variables: Record<string, string>): Promise<boolean> {
  const path = interpolate(step.path, variables);
  if (!operationMatches(step.route, path)) throw new Error("Request path does not match its declared API route");
  const multipart: Record<string, string | { name: string; mimeType: string; buffer: Buffer }> = {};
  let uploadBytes = 0;
  if (Object.keys(step.multipart ?? {}).length > 32) throw new Error("Upload has too many fields");
  for (const [name, part] of Object.entries(step.multipart ?? {})) {
    if (typeof part === "string") multipart[name] = interpolate(part, variables);
    else {
      const file = interpolate(part.file, variables);
      uploadBytes += statSync(file).size;
      if (uploadBytes > 64 * 1024 * 1024) throw new Error("Upload fixtures exceed 64 MiB in aggregate");
      multipart[name] = { name: file.split("/").pop() ?? "fixture", mimeType: part.mime_type, buffer: readFileSync(file) };
    }
  }
  const response = await context.request.fetch(localUrl(campaign.base_url, path), {
    method: step.method, timeout: 15000, maxRedirects: 0,
    headers: Object.fromEntries(Object.entries(step.headers ?? {}).map(([key, value]) => [key, interpolate(value, variables)])),
    ...(step.body !== undefined ? { data: expandJson(step.body, variables) } : {}),
    ...(step.multipart ? { multipart } : {}),
  });
  try {
    if (response.status() !== step.status) throw new Error(`Unexpected status for ${step.method} ${step.route}`);
    const body = await response.body();
    if (body.length > RESPONSE_LIMIT || body.length < (step.minimum_bytes ?? 0)) throw new Error("Response violated body size assertion");
    if (step.content_type && !response.headers()["content-type"]?.startsWith(step.content_type)) throw new Error("Response content type mismatch");
    if (step.json_pointer !== undefined || step.capture || (body.length > 0 && response.headers()["content-type"]?.includes("json"))) {
      const json: unknown = JSON.parse(body.toString("utf8"));
      if (step.json_pointer !== undefined && !isDeepStrictEqual(pointer(json, step.json_pointer), expandJson(step.equals ?? null, variables)))
        throw new Error(`Semantic response assertion failed for ${step.method} ${step.route}`);
      if (step.status < 400 && typeof json === "object" && json && "success" in json && json.success === false)
        throw new Error("An error envelope cannot establish successful feature coverage");
      for (const [name, source] of Object.entries(step.capture ?? {})) {
        const captured = pointer(json, source);
        if (typeof captured !== "string" && typeof captured !== "number") throw new Error("Capture must resolve to a scalar fixture value");
        const value = String(captured);
        if (value.length > 4096 || (!(name in variables) && Object.keys(variables).length >= 1024))
          throw new Error("Captured fixture variables exceeded bounds");
        variables[name] = value;
      }
    }
    return step.status < 400;
  } finally { await response.dispose(); }
}
export async function execute(
  context: BrowserContext, page: Page, campaign: Campaign, step: Step,
  variables: Record<string, string>, operations: Set<string>,
): Promise<void> {
  if (step.kind === "request") {
    if (await requestStep(context, campaign, step, variables)) operations.add(`${step.method} ${step.route}`);
  } else if (step.kind === "browser") {
    await page.goto(localUrl(campaign.base_url, interpolate(step.path, variables)), { waitUntil: "domcontentloaded" });
    for (const action of step.actions) {
      const locator = page.locator(action.selector);
      if (action.kind === "click") await locator.click();
      else if (action.kind === "fill") await locator.fill(interpolate(action.value, variables));
      else if (action.kind === "upload") await locator.setInputFiles(interpolate(action.file, variables));
      else if (action.kind === "press") await locator.press(action.key);
    }
    await expect(page.locator(step.selector)).toBeVisible();
    if (step.text) await expect(page.locator(step.selector)).toContainText(interpolate(step.text, variables));
  } else if (step.kind === "websocket") {
    await page.goto(campaign.base_url, { waitUntil: "domcontentloaded" });
    const url = localUrl(campaign.base_url, step.path).replace(/^https:/, "wss:");
    const succeeded = await page.evaluate(({ url, sendJson, expected }) => new Promise<boolean>((resolve) => {
      const socket = new WebSocket(url);
      let index = 0;
      const timer = setTimeout(() => { socket.close(); resolve(false); }, 15000);
      const finish = (result: boolean) => { clearTimeout(timer); socket.close(); resolve(result); };
      socket.onopen = () => { for (const message of JSON.parse(sendJson) as unknown[]) socket.send(JSON.stringify(message)); };
      socket.onerror = () => finish(false);
      socket.onmessage = (event: MessageEvent<unknown>) => {
        if (typeof event.data !== "string" || event.data.length > 1024 * 1024) { finish(false); return; }
        if (event.data.includes(expected[index] ?? "\0")) index += 1;
        if (index === expected.length) finish(true);
      };
    }), { url, sendJson: JSON.stringify(step.send.map((message) => expandJson(message, variables))), expected: step.expect.map((text) => interpolate(text, variables)) });
    if (!succeeded) throw new Error("WebSocket did not produce its expected protocol replies");
  } else if (step.kind === "command") {
    await hook(step.argv.map((value) => interpolate(value, variables)), required("CYHDEV_OPT_RUNTIME"));
  }
}
