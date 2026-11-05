import { expect, test, type Page, type WebSocketRoute } from "@playwright/test";
import { installApiMocks } from "./fixtures";

const NOW = "2026-09-03T08:00:00.000Z";
const uuid = (n: number) => `0198f4d0-aaaa-7000-8000-${String(n).padStart(12, "0")}`;
const author = (name: string, id: number) => ({
  public_user_id: uuid(id),
  display_name: name,
  country_code: 840,
  profile_picture_url: null,
  is_deleted: false,
});
const topic = (id: number, title: string, isPinned: boolean) => ({
  topic_id: uuid(id),
  author: author("opener", 900),
  title,
  body: `Body of ${title}`,
  content_state: "visible",
  access_state: "open",
  is_pinned: isPinned,
  revision: 1,
  reply_count: 2,
  created_at: NOW,
  updated_at: NOW,
  last_activity_at: NOW,
  edited_at: null,
});
const reply = (id: number, name: string, authorId: number, body: string) => ({
  reply_id: uuid(id),
  topic_id: uuid(3),
  author: author(name, authorId),
  body,
  content_state: "visible",
  revision: 1,
  created_at: NOW,
  updated_at: NOW,
  edited_at: null,
});

async function mockForum(page: Page, searches: string[] = []): Promise<void> {
  await page.route("**/api/forum/topics**", (route) => {
    const url = new URL(route.request().url());
    if (url.pathname !== "/api/forum/topics") {
      return route.fulfill({
        json: {
          success: true,
          meta: {},
          data: {
            is_subscribed: false,
            next_reply_cursor: null,
            topic: topic(3, "Thread title", false),
            replies: [
              reply(11, "first", 901, "A reply with https://example.com/page."),
              reply(12, "opener", 900, "> first wrote:\n> A reply\n\nThe answer."),
            ],
          },
        },
      });
    }
    const q = url.searchParams.get("search");
    if (q) searches.push(q);
    return route.fulfill({
      json: {
        success: true,
        meta: {},
        data: {
          next_cursor: null,
          topics: q
            ? [topic(3, "Matching topic", false)]
            : [topic(1, "Pinned rules", true), topic(3, "Thread title", false)],
        },
      },
    });
  });
}

const chatMessage = (index: number) => ({
  live_chat_message_id: uuid(100 + index),
  room_key: "main",
  user_id: null,
  sender_kind: 2,
  sender_display_name: "Guest",
  sender_country_flag: null,
  user_profile_picture_url: null,
  message_body: `Message ${index}`,
  message_created_at: NOW,
  message_edited_at: null,
  message_deleted_at: null,
});

/** Greets each connection once, as the server does; the first frame and no more. */
async function mockChat(
  page: Page,
  onConnect: (socket: WebSocketRoute) => void = () => {},
): Promise<void> {
  await page.routeWebSocket("**/ws/live-chat", (socket) => {
    socket.send(
      JSON.stringify({
        type: "hello",
        actor: {
          actor_key: { type: "guest", value: "0a1b2c3d4e5f60718293a4b5c6d7e8f9" },
          sender_kind: 2,
          user_id: null,
          display_name: "Guest",
          country_flag: null,
          user_profile_picture_url: null,
        },
        recent_messages: Array.from({ length: 30 }, (_, index) => chatMessage(index)),
        connected_count: 2,
      }),
    );
    onConnect(socket);
  });
}

/**
 * The active accent's lowest contrast on the page backgrounds in one mode, and
 * what a named swatch previews there, read from the real cascade.
 */
function accentReport(page: Page, dark: boolean, swatch: string) {
  return page.evaluate(
    ({ dark, swatch }) => {
      const root = document.documentElement;
      root.classList.toggle("dark", dark);
      root.classList.toggle("light", !dark);
      const token = (element: Element, property: string) =>
        getComputedStyle(element).getPropertyValue(property).trim();
      const luminance = (hex: string) => {
        const channel = (offset: number) => {
          const value = Number.parseInt(hex.slice(offset, offset + 2), 16) / 255;
          return value <= 0.03928 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
        };
        return 0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5);
      };
      const contrast = (first: string, second: string) => {
        const a = luminance(first);
        const b = luminance(second);
        return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
      };
      const accent = token(root, "--accent");
      const element = document.querySelector(`.accent-swatch[data-swatch="${swatch}"]`);
      return {
        accent,
        swatch: element ? token(element, "--swatch") : null,
        lowest: Math.min(
          ...["--paper", "--surface", "--accent-soft"].map((background) =>
            contrast(accent, token(root, background)),
          ),
        ),
      };
    },
    { dark, swatch },
  );
}

test.describe("appearance controls", () => {
  test("the desktop header offers accent and animation choices", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await page.goto("/");
    const trigger = page.getByRole("button", { name: "Appearance" });
    await trigger.click();
    const panel = page.getByRole("dialog", { name: "Appearance" });
    await panel.getByRole("radio", { name: "Cobalt" }).check({ force: true });
    await expect(page.locator("html")).toHaveAttribute("data-accent", "cobalt");
    const motion = panel.getByRole("switch", { name: "Animations" });
    await expect(motion).toHaveAttribute("aria-checked", "true");
    await motion.click();
    await expect(page.locator("html")).toHaveAttribute("data-motion", "off");
    await page.keyboard.press("Escape");
    await expect(panel).toBeHidden();
    await expect(trigger).toBeFocused();
    await page.reload();
    await expect(page.locator(".home-hero")).toBeVisible();
    await expect(page.locator("html")).toHaveAttribute("data-accent", "cobalt");
    await expect(page.locator("html")).toHaveAttribute("data-motion", "off");
  });

  test("every preset survives a reload, previews itself, and stays readable", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await page.goto("/");
    const trigger = page.getByRole("button", { name: "Appearance" });
    await trigger.click();
    const names = await page
      .getByRole("dialog", { name: "Appearance" })
      .getByRole("radio")
      .evaluateAll((inputs) => inputs.map((input) => (input as HTMLInputElement).value));
    expect(names).toEqual(["amber", "lagoon", "cobalt", "orchid", "graphite", "custom"]);

    // Amber is the default and keeps its historical values.
    for (const name of ["lagoon", "cobalt", "orchid", "graphite"]) {
      await page.evaluate((value) => localStorage.setItem("accent", value), name);
      await page.reload();
      await expect(page.locator("html")).toHaveAttribute("data-accent", name);
      await trigger.click();
      for (const dark of [false, true]) {
        const report = await accentReport(page, dark, name);
        expect(report.swatch, `${name} dark=${dark} swatch`).toBe(report.accent);
        expect(report.lowest, `${name} dark=${dark} contrast`).toBeGreaterThanOrEqual(4.5);
      }
    }
  });

  test("a typed RGB value becomes the accent, persists, and can be reselected", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await page.goto("/");
    const html = page.locator("html");
    const source = () =>
      html.evaluate((root) => getComputedStyle(root).getPropertyValue("--custom-accent-source").trim());
    const panel = page.getByRole("dialog", { name: "Appearance" });
    const field = panel.getByRole("textbox", { name: "RGB value" });
    const custom = panel.getByRole("radio", { name: "Custom" });

    await page.getByRole("button", { name: "Appearance" }).click();
    await expect(field).toHaveCount(0);
    await custom.check({ force: true });
    await expect(html).toHaveAttribute("data-accent", "custom");
    await field.fill("255, 120, 40");
    await expect.poll(source).toBe("#ff7828");
    await field.fill("rgb(10 90 200)");
    await expect.poll(source).toBe("#0a5ac8");
    await field.fill("#ff7828");
    await expect.poll(source).toBe("#ff7828");

    await page.reload();
    await expect(html).toHaveAttribute("data-accent", "custom");
    expect(await source()).toBe("#ff7828");

    await page.getByRole("button", { name: "Appearance" }).click();
    await expect(custom).toBeChecked();
    await expect(field).toHaveValue("255, 120, 40");
    await panel.getByRole("radio", { name: "Cobalt" }).check({ force: true });
    await expect(html).toHaveAttribute("data-accent", "cobalt");
    await expect(field).toHaveCount(0);
    await custom.check({ force: true });
    await expect(html).toHaveAttribute("data-accent", "custom");
    await expect(field).toHaveValue("255, 120, 40");
  });

  test("a custom color is applied before the application loads", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await page.goto("/");
    await page.getByRole("button", { name: "Appearance" }).click();
    const panel = page.getByRole("dialog", { name: "Appearance" });
    await panel.getByRole("radio", { name: "Custom" }).check({ force: true });
    await panel.getByRole("textbox", { name: "RGB value" }).fill("10, 90, 200");
    await page.route("**/src/index.tsx*", (route) => route.abort());
    await page.reload();
    const html = page.locator("html");
    await expect(html).toHaveAttribute("data-accent", "custom");
    expect(
      await html.evaluate((root) => root.style.getPropertyValue("--custom-accent-source")),
    ).toBe("#0a5ac8");
  });

  test("any custom color stays readable, and says when it was adjusted", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await page.goto("/");
    await page.getByRole("button", { name: "Appearance" }).click();
    const panel = page.getByRole("dialog", { name: "Appearance" });
    await panel.getByRole("radio", { name: "Custom" }).check({ force: true });
    const field = panel.getByRole("textbox", { name: "RGB value" });
    for (const color of ["255, 255, 0", "255, 255, 255", "0, 0, 0", "128, 128, 128", "0, 255, 0"]) {
      await field.fill(color);
      for (const dark of [false, true]) {
        const report = await accentReport(page, dark, "custom");
        expect(report.lowest, `${color} dark=${dark}`).toBeGreaterThanOrEqual(4.5);
      }
    }
    await accentReport(page, false, "custom");
    await field.fill("255, 255, 0");
    await expect(panel.locator(".accent-custom-note")).toContainText("Adjusted");
    await field.fill("10, 90, 200");
    await expect(panel.locator(".accent-custom-note")).toHaveCount(0);
  });

  test("an unreadable RGB value is refused only once the field is left", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await page.goto("/");
    await page.getByRole("button", { name: "Appearance" }).click();
    const panel = page.getByRole("dialog", { name: "Appearance" });
    await panel.getByRole("radio", { name: "Custom" }).check({ force: true });
    const field = panel.getByRole("textbox", { name: "RGB value" });
    await field.fill("10, 90, 200");
    await field.fill("10, 90");
    await expect(field).not.toHaveAttribute("aria-invalid", "true");
    await field.blur();
    await expect(field).toHaveAttribute("aria-invalid", "true");
    await expect(panel.getByRole("alert")).toBeVisible();
    expect(
      await page.locator("html").evaluate((root) => root.style.getPropertyValue("--custom-accent-source")),
    ).toBe("#0a5ac8");
    await field.fill("255, 120, 40");
    await expect(panel.getByRole("alert")).toHaveCount(0);
  });

  test("the drawer offers the custom color without overflowing a small phone", async ({ page }) => {
    await page.setViewportSize({ width: 320, height: 568 });
    await installApiMocks(page, "logged-out");
    await page.goto("/");
    await page.locator("#mobile-navigation-trigger").click();
    const dialog = page.getByRole("dialog");
    // The visible swatch, not its hidden radio: a click on it waits out the drawer's slide.
    await dialog.locator('.accent-swatch[data-swatch="custom"]').click();
    await expect(dialog.getByRole("radio", { name: "Custom" })).toBeChecked();
    const field = dialog.getByRole("textbox", { name: "RGB value" });
    await field.fill("255, 255, 0");
    await expect(dialog.locator(".accent-custom-note")).toBeVisible();
    const box = await field.boundingBox();
    expect((box?.x ?? 0) + (box?.width ?? 0)).toBeLessThanOrEqual(320);
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(320);
  });

  test("tabbing out of the panel closes it", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await page.goto("/");
    await page.getByRole("button", { name: "Appearance" }).click();
    const panel = page.getByRole("dialog", { name: "Appearance" });
    await panel.getByRole("switch", { name: "Animations" }).focus();
    await page.keyboard.press("Tab");
    await expect(panel).toBeHidden();
  });

  test("an outside press closes the panel", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await page.goto("/");
    await page.getByRole("button", { name: "Appearance" }).click();
    await page.locator(".home-hero-title").click();
    await expect(page.getByRole("dialog", { name: "Appearance" })).toBeHidden();
  });

  test("animations ignore the OS preference and follow the site switch", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await page.goto("/");
    const board = page.locator(".home-board");
    const duration = () =>
      board.evaluate((element) => Number.parseFloat(getComputedStyle(element).animationDuration));
    await expect(page.locator("html")).not.toHaveAttribute("data-motion", "off");
    expect(await duration()).toBeGreaterThan(0.1);
    await page.getByRole("button", { name: "Appearance" }).click();
    await page.getByRole("switch", { name: "Animations" }).click();
    expect(await duration()).toBeLessThan(0.001);
  });

  test("the drawer offers the animation switch on phones", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await installApiMocks(page, "logged-out");
    await page.goto("/");
    await page.locator("#mobile-navigation-trigger").click();
    const toggle = page.getByRole("dialog").getByRole("switch", { name: "Animations" });
    await toggle.click();
    await expect(page.locator("html")).toHaveAttribute("data-motion", "off");
  });

  for (const mode of ["logged-out", "authenticated"] as const) {
    test(`the header has no login status dot when ${mode}`, async ({ page }) => {
      await page.setViewportSize({ width: 1440, height: 900 });
      await installApiMocks(page, mode);
      await page.goto("/");
      await expect(page.locator(".home-hero")).toBeVisible();
      await expect(page.locator(".site-header :is(.bg-danger, .bg-ok)")).toHaveCount(0);
    });
  }
});

test.describe("home board", () => {
  test("chat sits left of posts, photographs, and notes on desktop", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await mockChat(page);
    await page.goto("/");
    await expect(page.locator(".live-chat-message").first()).toBeVisible();
    const box = async (selector: string) => {
      const value = await page.locator(selector).boundingBox();
      if (!value) throw new Error(`${selector} has no box`);
      return value;
    };
    const chat = await box(".home-panel--chat");
    const posts = await box(".home-panel--posts");
    const photos = await box(".home-panel--photos");
    const notes = await box(".home-panel--notes");
    expect(chat.x + chat.width).toBeLessThanOrEqual(posts.x);
    expect(posts.x + posts.width).toBeLessThanOrEqual(photos.x);
    expect(notes.y).toBeGreaterThanOrEqual(posts.y + posts.height);
    expect(notes.x).toBe(posts.x);
    expect(chat.x).toBeLessThan(32);
    await expect(page.locator(".live-chat-composer")).toBeInViewport();
  });

  for (const [width, height] of [
    [800, 600],
    [1024, 768],
    [1280, 720],
    [1440, 900],
    [1920, 1080],
    [2560, 1440],
  ] as const) {
    test(`chat ends level with the notes at ${width}x${height}`, async ({ page }) => {
      await page.setViewportSize({ width, height });
      await installApiMocks(page, "logged-out");
      await mockChat(page);
      await page.goto("/");
      await expect(page.locator(".live-chat-message").first()).toBeAttached();
      // Where the panels stack, chat stays in view and meets them at the page's end.
      await page.evaluate(() => window.scrollTo(0, document.documentElement.scrollHeight));
      const bottom = async (selector: string) => {
        const value = await page.locator(selector).boundingBox();
        if (!value) throw new Error(`${selector} has no box`);
        return value.y + value.height;
      };
      await expect
        .poll(async () => Math.abs((await bottom(".live-chat-panel")) - (await bottom(".home-panel--notes"))))
        .toBeLessThan(1);
      // Clear of the fixed footer, which wraps taller at some widths.
      const footer = await page.locator('[data-site-bar="bottom"]').boundingBox();
      if (!footer) throw new Error("footer has no box");
      expect(await bottom(".live-chat-composer")).toBeLessThanOrEqual(footer.y);
      expect(await bottom(".home-panel--notes")).toBeLessThanOrEqual(footer.y);
    });
  }

  const blogPosts = (count: number) =>
    Array.from({ length: count }, (_, index) => ({
      post_id: uuid(200 + index),
      post_slug: `post-${index}`,
      post_title: `Post ${index}`,
      post_created_at: NOW,
      post_is_published: true,
    }));

  async function boardWithPosts(page: Page, count: number): Promise<void> {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    // Box reads must not race the board's entrance animation.
    await page.addInitScript(() => localStorage.setItem("motion", "off"));
    await page.route("**/api/blog/posts?**", (route) =>
      route.fulfill({
        json: { success: true, meta: {}, data: { available_pages: 1, posts: blogPosts(count) } },
      }),
    );
    await page.goto("/");
  }

  test("a lone post stays a thin row in a box that matches the photographs", async ({ page }) => {
    await boardWithPosts(page, 1);
    const list = await page.locator(".home-post-list").boundingBox();
    const row = await page.locator(".home-post").boundingBox();
    const photos = await page.locator(".home-photo-grid").boundingBox();
    if (!list || !row || !photos) throw new Error("board panels have no box");
    expect(row.height).toBeLessThan(56);
    expect(Math.abs(list.y + list.height - (photos.y + photos.height))).toBeLessThan(1);
  });

  test("a full list of posts shares the box evenly", async ({ page }) => {
    await boardWithPosts(page, 5);
    const rows = page.locator(".home-post");
    await expect(rows).toHaveCount(5);
    const list = await page.locator(".home-post-list").boundingBox();
    const row = await rows.first().boundingBox();
    if (!list || !row) throw new Error("board panels have no box");
    expect(row.height).toBeGreaterThan(list.height / 5 - 2);
  });

  test("no posts says so instead of showing an empty box", async ({ page }) => {
    await boardWithPosts(page, 0);
    await expect(page.locator(".home-panel--posts .home-empty")).toBeVisible();
    await expect(page.locator(".home-post-list")).toHaveCount(0);
  });

  test("a failed posts or photographs request leaves the rest of the page working", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await page.route("**/api/blog/posts?**", (route) => route.fulfill({ status: 500, json: {} }));
    await page.route("**/api/photographs**", (route) => route.fulfill({ status: 500, json: {} }));
    await page.route("**/api/photography**", (route) => route.fulfill({ status: 500, json: {} }));
    await page.goto("/");
    await expect(page.locator(".home-panel--posts .home-empty")).toBeVisible();
    await expect(page.locator(".home-photo-empty")).toBeVisible();
    await expect(page.locator(".home-hero-title")).toBeVisible();
    await expect(page.locator(".home-showcase-card")).toHaveCount(6);
  });

  test("phones stack notes last and defer chat until it is near", async ({ page }) => {
    await page.setViewportSize({ width: 320, height: 568 });
    await installApiMocks(page, "logged-out");
    let opened = false;
    await page.routeWebSocket("**/ws/live-chat", () => {
      opened = true;
    });
    await page.goto("/");
    const top = async (selector: string) =>
      (await page.locator(selector).boundingBox())?.y ?? Number.NaN;
    await expect(page.locator(".home-panel--notes")).toBeAttached();
    expect(await top(".home-panel--posts")).toBeLessThan(await top(".home-panel--photos"));
    expect(await top(".home-panel--photos")).toBeLessThan(await top(".home-panel--chat"));
    expect(await top(".home-panel--chat")).toBeLessThan(await top(".home-panel--notes"));
    await page.waitForTimeout(300);
    expect(opened).toBe(false);
    await page.locator(".home-panel--chat").scrollIntoViewIfNeeded();
    await expect.poll(() => opened).toBe(true);
  });

  test("a chat chunk that fails to load leaves the page working", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await page.route("**/components/LiveChatPanel*", (route) => route.abort());
    await page.goto("/");
    await expect(page.locator(".home-panel--chat .home-empty")).toBeVisible();
    await expect(page.locator(".home-hero-title")).toBeVisible();
    await expect(page.locator(".home-panel--posts")).toBeVisible();
  });

  test("loading chat scrolls its own list, not the page", async ({ page }) => {
    await page.setViewportSize({ width: 800, height: 500 });
    await installApiMocks(page, "logged-out");
    await mockChat(page);
    await page.goto("/");
    const list = page.locator(".live-chat-messages");
    await expect(page.locator(".live-chat-message").first()).toBeAttached();
    await expect
      .poll(() => list.evaluate((element) => element.scrollTop))
      .toBeGreaterThan(0);
    expect(await page.evaluate(() => window.scrollY)).toBe(0);
  });
});

test.describe("live chat across routes", () => {
  test("moving between the home and chat pages keeps history and presence", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    const sockets: WebSocketRoute[] = [];
    await mockChat(page, (socket) => sockets.push(socket));
    await page.goto("/");
    const newest = page.locator(".live-chat-message").last();
    const status = page.locator(".live-chat-status");
    await expect(newest).toContainText("Message 29");
    await expect(status).toContainText("2");

    // The first visit may wait on the route's chunk; later ones reuse the socket.
    for (let round = 0; round < 3; round += 1) {
      await page.locator(".live-chat-open").click();
      await expect(page).toHaveURL(/\/live-chat$/);
      await expect(newest).toContainText("Message 29");
      await expect(status).toContainText("2");
      await page.locator(".site-header-brand").click();
      await expect(page.locator(".home-hero")).toBeVisible();
      await expect(newest).toContainText("Message 29");
      await expect(status).toContainText("2");
    }
    expect(sockets.length).toBeLessThan(7);

    // What arrives on the open socket is still there on the next route.
    const socket = sockets.at(-1);
    if (!socket) throw new Error("no chat socket opened");
    socket.send(JSON.stringify({ type: "message", message: chatMessage(30) }));
    socket.send(JSON.stringify({ type: "presence", connected_count: 7 }));
    await expect(newest).toContainText("Message 30");
    socket.send(
      JSON.stringify({ type: "message_deleted", live_chat_message_id: uuid(129), room_key: "main" }),
    );
    await page.locator(".live-chat-open").click();
    await expect(page).toHaveURL(/\/live-chat$/);
    await expect(newest).toContainText("Message 30");
    await expect(status).toContainText("7");
    await expect(page.locator(".live-chat-message", { hasText: "Message 29" })).toHaveCount(0);
  });

  test("signing out reconnects chat, since the server fixed the identity at connect", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "authenticated");
    let connections = 0;
    await mockChat(page, () => {
      connections += 1;
    });
    await page.goto("/");
    await expect(page.locator(".live-chat-message").last()).toContainText("Message 29");
    expect(connections).toBe(1);
    await page.locator(".menu-toggle").click();
    await page.getByRole("menuitem", { name: "Logout" }).click();
    await expect.poll(() => connections).toBe(2);
    await expect(page.locator(".live-chat-message").last()).toContainText("Message 29");
  });

  test("a full compact window still follows new messages", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    let socket: WebSocketRoute | undefined;
    await mockChat(page, (opened) => {
      socket = opened;
    });
    await page.goto("/");
    const list = page.locator(".live-chat-messages");
    await expect(page.locator(".live-chat-message").last()).toContainText("Message 29");
    for (let index = 30; index < 60; index += 1) {
      socket?.send(JSON.stringify({ type: "message", message: chatMessage(index) }));
    }
    await expect(page.locator(".live-chat-message").last()).toContainText("Message 59");
    await expect
      .poll(() =>
        list.evaluate((element) => element.scrollHeight - element.scrollTop - element.clientHeight),
      )
      .toBeLessThan(2);
  });
});

test.describe("forum", () => {
  test("the topic table groups pinned topics and links whole rows", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await mockForum(page);
    await page.goto("/forum");
    await expect(page.getByRole("heading", { name: "Pinned topics" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Latest topics" })).toBeVisible();
    await expect(page.locator(".forum-topic-row")).toHaveCount(2);
    await expect(page.locator(".forum-notice").getByRole("link", { name: "Sign in" })).toBeVisible();
    await page.locator(".forum-topic-row__link", { hasText: "Thread title" }).click();
    await expect(page).toHaveURL(new RegExp(`/forum/${uuid(3)}$`));
  });

  test("search is linkable and survives going back", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    const searches: string[] = [];
    await mockForum(page, searches);
    await page.goto("/forum");
    const input = page.locator(".forum-search input");
    await expect(page.locator(".forum-topic-row")).toHaveCount(2);
    await input.fill("matching");
    await input.press("Enter");
    await expect(page).toHaveURL(/\?q=matching$/);
    await expect(page.locator(".forum-topic-row")).toHaveCount(1);
    await expect(page.locator(".forum-search-summary")).toContainText("matching");
    await expect(input).toBeFocused();
    await page.goBack();
    await expect(page.locator(".forum-topic-row")).toHaveCount(2);
    await expect(input).toHaveValue("");
    await page.goForward();
    await expect(input).toHaveValue("matching");
    await page.locator(".forum-search-summary button").click();
    await expect(page).toHaveURL(/\/forum$/);
    expect(searches.every((search) => search === "matching")).toBe(true);
  });

  test("a thread numbers its posts, marks the author, and quotes into the reply", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "authenticated");
    await mockForum(page);
    await page.goto(`/forum/${uuid(3)}`);
    const posts = page.locator(".forum-post");
    await expect(posts).toHaveCount(3);
    await expect(posts.nth(0).locator(".forum-post__number")).toHaveText("#1");
    await expect(posts.nth(2).locator(".forum-post__number")).toHaveText("#3");
    await expect(posts.nth(1).locator(".forum-tag")).toHaveCount(0);
    await expect(posts.nth(2).locator(".forum-tag")).toBeVisible();
    await expect(posts.nth(2).locator("blockquote")).toContainText("A reply");
    const link = posts.nth(1).getByRole("link", { name: "https://example.com/page" });
    await expect(link).toHaveAttribute("rel", /nofollow/);
    await posts.nth(1).getByRole("button", { name: "Quote" }).click();
    const composer = page.locator("#forum-new-reply");
    await expect(composer).toHaveValue(/^> first wrote:\n> A reply with/);
    await expect(composer).toBeFocused();
  });

  test("a failed search does not leave the last query's rows or cursor", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await page.route("**/api/forum/topics?**", (route) => {
      const search = new URL(route.request().url()).searchParams.get("search");
      if (search === "broken") return route.fulfill({ status: 500, json: {} });
      return route.fulfill({
        json: {
          success: true,
          meta: {},
          data: {
            next_cursor: { last_activity_at: NOW, topic_id: uuid(3) },
            topics: [topic(3, "Thread title", false)],
          },
        },
      });
    });
    await page.goto("/forum");
    await expect(page.locator(".forum-topic-row")).toHaveCount(1);
    await expect(page.locator(".forum-pagination")).toBeVisible();
    await page.goto("/forum?q=broken");
    await expect(page.locator(".forum-topic-row")).toHaveCount(0);
    await expect(page.locator(".forum-pagination")).toHaveCount(0);
    await page.goto("/forum");
    await page.locator(".forum-search input").fill("broken");
    await page.locator(".forum-search input").press("Enter");
    await expect(page).toHaveURL(/\?q=broken$/);
    await expect(page.locator(".forum-topic-row")).toHaveCount(0);
    await expect(page.locator(".forum-pagination")).toHaveCount(0);
  });

  test("a hidden topic's breadcrumb does not claim to be loading", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    await page.route("**/api/forum/topics/**", (route) =>
      route.fulfill({
        json: {
          success: true,
          meta: {},
          data: {
            is_subscribed: false,
            next_reply_cursor: null,
            topic: { ...topic(3, "", false), title: null, body: null, content_state: "hidden" },
            replies: [],
          },
        },
      }),
    );
    await page.goto(`/forum/${uuid(3)}`);
    await expect(page.locator(".forum-post")).toHaveCount(1);
    await expect(page.locator(".forum-breadcrumb__current")).toHaveText(
      "This content is not publicly available.",
    );
  });

  test("replies by other deleted accounts are not marked as the topic author", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await installApiMocks(page, "logged-out");
    const nobody = {
      ...author("deleted", 0),
      public_user_id: "00000000-0000-0000-0000-000000000000",
      is_deleted: true,
    };
    await page.route("**/api/forum/topics/**", (route) =>
      route.fulfill({
        json: {
          success: true,
          meta: {},
          data: {
            is_subscribed: false,
            next_reply_cursor: null,
            topic: { ...topic(3, "Thread title", false), author: nobody },
            replies: [{ ...reply(11, "deleted", 0, "Body"), author: nobody }],
          },
        },
      }),
    );
    await page.goto(`/forum/${uuid(3)}`);
    const posts = page.locator(".forum-post");
    await expect(posts).toHaveCount(2);
    await expect(posts.nth(1).locator(".forum-tag")).toHaveCount(0);
  });

  test("moderation controls stay collapsed until asked for", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await installApiMocks(page, "superuser");
    await mockForum(page);
    await page.goto(`/forum/${uuid(3)}`);
    const opening = page.locator(".forum-post").first();
    await expect(opening.getByRole("button", { name: "Moderate" })).toBeVisible();
    await expect(page.locator(".forum-moderation__form")).toHaveCount(0);
    await opening.getByRole("button", { name: "Moderate" }).click();
    const form = opening.locator(".forum-moderation__form");
    await expect(form).toBeVisible();
    const box = await form.boundingBox();
    expect((box?.x ?? 0) + (box?.width ?? 0)).toBeLessThanOrEqual(390);
  });

  test("guests get a sign-in prompt instead of a reply box", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await installApiMocks(page, "logged-out");
    await mockForum(page);
    await page.goto(`/forum/${uuid(3)}`);
    await expect(page.locator(".forum-post")).toHaveCount(3);
    await expect(page.locator("#forum-new-reply")).toHaveCount(0);
    await expect(page.locator(".forum-composer-notice").getByRole("link")).toHaveAttribute("href", "/login");
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
  });
});
