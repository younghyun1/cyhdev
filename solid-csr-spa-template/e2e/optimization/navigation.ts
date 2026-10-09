import type { Response } from "@playwright/test";

/** Report admission failures without copying capability URLs or response bodies into diagnostics. */
export function assertPageResponse(pattern: string, response: Pick<Response, "ok" | "status" | "headers"> | null): void {
  if (!response) throw new Error(`Page response missing for ${pattern}`);
  if (response.ok()) return;
  const retryAfter = response.headers()["retry-after"];
  const retryHint = retryAfter && /^\d{1,8}$/.test(retryAfter) ? `; Retry-After ${retryAfter}s` : "";
  throw new Error(`Page response failed for ${pattern}: HTTP ${response.status()}${retryHint}`);
}
