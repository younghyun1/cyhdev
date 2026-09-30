import { type Browser, type BrowserContext } from "@playwright/test";
import { requestStep } from "./steps";
import type { Actor, Campaign } from "./model";

/** Server sessions are in memory, so every server start needs actual fixture logins. */
export async function actorContext(browser: Browser, campaign: Campaign, actor: Actor): Promise<BrowserContext> {
  const context = await browser.newContext({ ignoreHTTPSErrors: true, permissions: ["camera", "microphone"] });
  try {
    const login = campaign.actors[actor].login;
    if (actor !== "anonymous") {
      if (!login || !await requestStep(context, campaign, login, campaign.parameters)) throw new Error("Fixture login failed");
      const response = await context.request.get(`${campaign.base_url}/api/auth/is-superuser`, { timeout: 15000 });
      try {
        const body: unknown = await response.json();
        const data = typeof body === "object" && body && "data" in body ? body.data : null;
        const isAdmin = typeof data === "object" && data && "is_superuser" in data ? data.is_superuser : null;
        if (!response.ok() || isAdmin !== (actor === "admin")) throw new Error("Fixture actor has incorrect server authority");
      } finally { await response.dispose(); }
    }
    return context;
  } catch (error: unknown) { await context.close(); throw error; }
}
