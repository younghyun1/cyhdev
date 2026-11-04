import { expect, test, type Page } from "@playwright/test";
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

async function mockChat(page: Page): Promise<void> {
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
        recent_messages: Array.from({ length: 30 }, (_, index) => ({
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
        })),
        connected_count: 2,
      }),
    );
  });
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
