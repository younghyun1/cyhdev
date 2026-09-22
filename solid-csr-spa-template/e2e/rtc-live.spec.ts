import { expect, test, type Page } from "@playwright/test";
import { resolve } from "node:path";

const origin = process.env.RTC_SMOKE_URL;
test.skip(!origin, "Requires an explicitly configured local RTC backend and two test accounts");
test.use({ launchOptions: { args: ["--use-fake-device-for-media-stream", "--use-fake-ui-for-media-stream", "--autoplay-policy=no-user-gesture-required"] } });

interface ObservedWindow extends Window {
  rtcSmokePeers: RTCPeerConnection[];
  rtcSmokeSockets: WebSocket[];
}

async function mediaCounters(page: Page) {
  return page.evaluate(async () => {
    const peer = (window as unknown as ObservedWindow).rtcSmokePeers.at(-1);
    const result = { audio: 0, video: 0, state: peer?.connectionState };
    if (!peer) return result;
    (await peer.getStats()).forEach((row) => {
      if (row.type !== "inbound-rtp") return;
      if (row.kind === "audio") result.audio += Number(row.bytesReceived ?? 0);
      if (row.kind === "video") result.video += Number(row.framesDecoded ?? 0);
    });
    return result;
  });
}

async function expectFreshMedia(page: Page) {
  await expect.poll(async () => (await mediaCounters(page)).state, { timeout: 30_000 }).toBe("connected");
  const baseline = await mediaCounters(page);
  await expect.poll(async () => (await mediaCounters(page)).audio, { timeout: 30_000 }).toBeGreaterThan(baseline.audio);
  await expect.poll(async () => (await mediaCounters(page)).video, { timeout: 30_000 }).toBeGreaterThan(baseline.video);
}

test("two browsers exchange media, toggle devices, leave, and reconnect", async ({ browser }, testInfo) => {
  test.setTimeout(120_000);
  if (!origin || !/^https:\/\/(127\.0\.0\.1|localhost):\d+$/.test(origin)) {
    throw new Error("RTC_SMOKE_URL must name an explicit loopback HTTPS backend");
  }
  const contexts = await Promise.all([0, 1].map(() => browser.newContext({
    baseURL: origin, ignoreHTTPSErrors: true, permissions: ["camera", "microphone"],
  })));
  try {
    const pages: Page[] = [];
    for (const [index, context] of contexts.entries()) {
      const login = await context.request.post("/api/auth/login", {
        headers: { Origin: origin },
        data: { user_email: `rtcsmoke${index === 0 ? "a" : "b"}@example.test`, user_password: "LocalSmokePass123" },
      });
      expect(login.status()).toBe(200);
      await context.addInitScript(() => {
        localStorage.setItem("ui_locale", "en-US");
        const observed = window as unknown as ObservedWindow;
        observed.rtcSmokePeers = [];
        observed.rtcSmokeSockets = [];
        const NativePeer = window.RTCPeerConnection;
        window.RTCPeerConnection = class extends NativePeer {
          constructor(configuration?: RTCConfiguration) {
            super(configuration);
            observed.rtcSmokePeers.push(this);
          }
        };
        const NativeSocket = window.WebSocket;
        window.WebSocket = class extends NativeSocket {
          constructor(url: string | URL, protocols?: string | string[]) {
            super(url, protocols);
            observed.rtcSmokeSockets.push(this);
          }
        };
      });
      const page = await context.newPage();
      pages.push(page);
      page.on("console", (entry) => { if (entry.type() === "error") console.error(entry.text()); });
      await page.goto("/live-chat");
      await page.waitForFunction(() => (window as unknown as ObservedWindow).rtcSmokeSockets.some((socket) => socket.url.includes("/ws/live-chat") && socket.readyState === WebSocket.OPEN));
      await page.getByRole("button", { name: "Join call", exact: true }).click();
      await expect(page.getByRole("button", { name: "Leave", exact: true })).toBeVisible();
    }
    const [first, second] = pages;
    if (!first || !second) throw new Error("Two browser contexts are required");
    for (const page of pages) {
      await expect.poll(async () => (await mediaCounters(page)).state, { timeout: 30_000 }).toBe("connected");
      await expect.poll(async () => (await mediaCounters(page)).audio, { timeout: 30_000 }).toBeGreaterThan(0);
      await expect.poll(async () => (await mediaCounters(page)).video, { timeout: 30_000 }).toBeGreaterThan(0);
      await expect(page.getByText("2 in call", { exact: true })).toBeVisible();
    }
    await first.getByRole("button", { name: "Mute", exact: true }).click();
    await expect(first.getByRole("button", { name: "Unmute", exact: true })).toBeVisible();
    await first.getByRole("button", { name: "Stop video", exact: true }).click();
    await expect(first.getByRole("button", { name: "Start video", exact: true })).toBeVisible();
    const enabled = await first.evaluate(() => (window as unknown as ObservedWindow).rtcSmokePeers.at(-1)?.getSenders().filter((sender) => sender.track).map((sender) => sender.track?.enabled));
    expect(enabled).toEqual([false, false]);
    await first.getByRole("button", { name: "Unmute", exact: true }).click();
    await first.getByRole("button", { name: "Start video", exact: true }).click();
    await first.getByRole("button", { name: "Leave", exact: true }).click();
    await expect(second.getByText("1 in call", { exact: true })).toBeVisible();
    await first.getByRole("button", { name: "Join call", exact: true }).click();
    for (const page of pages) await expectFreshMedia(page);
    await first.evaluate(() => (window as unknown as ObservedWindow).rtcSmokeSockets.find((socket) => socket.url.includes("/ws/live-chat") && socket.readyState === WebSocket.OPEN)?.close());
    await expect(first.getByRole("button", { name: "Join call", exact: true })).toBeVisible();
    await expect(second.getByText("1 in call", { exact: true })).toBeVisible();
    await first.waitForFunction(() => (window as unknown as ObservedWindow).rtcSmokeSockets.some((socket) => socket.url.includes("/ws/live-chat") && socket.readyState === WebSocket.OPEN));
    await first.getByRole("button", { name: "Join call", exact: true }).click();
    for (const [index, page] of pages.entries()) {
      await expectFreshMedia(page);
      await expect(page.getByText("2 in call", { exact: true })).toBeVisible();
      await expect.poll(() => page.locator("video").evaluateAll((videos) => videos.filter((video) => video.videoWidth > 0 && video.srcObject !== null).length)).toBe(2);
      const screenshot = resolve(import.meta.dirname, `../../target/rtc-smoke/participant-${index + 1}.png`);
      await page.screenshot({ path: screenshot });
      await testInfo.attach(`participant-${index + 1}`, { path: screenshot, contentType: "image/png" });
      console.log(`participant-${index + 1} media`, await mediaCounters(page));
    }
    for (const page of pages) await page.getByRole("button", { name: "Leave", exact: true }).click();
  } finally {
    await Promise.all(contexts.map((context) => context.close()));
  }
});
