import type { Page } from "@playwright/test";

export type AuthMode = "logged-out" | "authenticated" | "superuser";

const ID = "11111111-1111-4111-8111-111111111111";
const ID_2 = "22222222-2222-4222-8222-222222222222";
const NOW = "2026-09-03T08:00:00.000Z";
const IMAGE =
  "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";
const LONG_TEXT =
  "A mobile layout must contain this deliberately long unbroken identifier without document overflow: 12345678-1234-4234-8234-123456789012/path/that/keeps/going";

const api = (data: unknown) => ({
  success: true,
  data,
  meta: { time_to_process: "1ms" },
});

const user = {
  user_country: 840,
  user_email: "mobile.superuser.with.a.long.address@example.test",
  user_id: ID,
  user_is_email_verified: true,
  user_language: 1,
  user_name: "mobile-superuser",
  user_subdivision: null,
};

const postItem = {
  post_created_at: NOW,
  post_id: ID,
  post_is_published: true,
  post_published_at: NOW,
  post_share_count: 4,
  post_slug: "mobile-layout",
  post_summary: LONG_TEXT,
  post_tags: ["responsive", "solidjs"],
  post_title: LONG_TEXT,
  post_updated_at: NOW,
  post_view_count: 12,
  total_downvotes: 1,
  total_upvotes: 8,
  user_country_flag: "🇺🇸",
  user_id: ID,
  user_name: "mobile-superuser",
  user_profile_picture_url: IMAGE,
  vote_state: 2,
};

const photograph = (id: string, comment: string) => ({
  photograph_comments: comment,
  photograph_context: "photography",
  photograph_created_at: NOW,
  photograph_id: id,
  photograph_image_type: 1,
  photograph_is_on_cloud: true,
  photograph_lat: 39.7392,
  photograph_link: IMAGE,
  photograph_lon: -104.9903,
  photograph_shot_at: NOW,
  photograph_thumbnail_link: IMAGE,
  photograph_total_downvotes: 1,
  photograph_total_upvotes: 6,
  photograph_updated_at: NOW,
  photograph_view_count: 25,
  user_id: ID,
});

const photographComments = Array.from({ length: 8 }, (_, depth) => ({
  parent_photograph_comment_id:
    depth === 0 ? null : `photo-comment-${depth - 1}`,
  photograph_comment_content: `${LONG_TEXT} depth ${depth}`,
  photograph_comment_created_at: NOW,
  photograph_comment_id: `photo-comment-${depth}`,
  photograph_comment_total_downvotes: 0,
  photograph_comment_total_upvotes: 1,
  photograph_comment_updated_at: NOW,
  photograph_id: ID,
  user_country_flag: "🇺🇸",
  user_id: ID,
  user_name: "mobile-superuser",
  user_profile_picture_url: IMAGE,
  vote_state: 2,
}));

const blogComments = Array.from({ length: 8 }, (_, depth) => ({
  comment_content: `${LONG_TEXT} depth ${depth}`,
  comment_created_at: NOW,
  comment_id: `blog-comment-${depth}`,
  comment_updated_at: NOW,
  parent_comment_id: depth === 0 ? null : `blog-comment-${depth - 1}`,
  post_id: ID,
  total_downvotes: 0,
  total_upvotes: 1,
  user_country_flag: "🇺🇸",
  user_id: ID,
  user_name: "mobile-superuser",
  user_profile_picture_url: IMAGE,
  vote_state: 2,
}));

const forumTopic = {
  access_state: "open",
  author: {
    country_code: 840,
    display_name: "mobile-superuser",
    is_deleted: false,
    profile_picture_url: IMAGE,
    public_user_id: ID,
  },
  body: LONG_TEXT,
  content_state: "visible",
  created_at: NOW,
  edited_at: null,
  is_pinned: false,
  last_activity_at: NOW,
  reply_count: 0,
  revision: 1,
  title: LONG_TEXT,
  topic_id: ID,
  updated_at: NOW,
};

export async function installApiMocks(
  page: Page,
  authMode: AuthMode = "superuser",
): Promise<void> {
  await page.route("https://tile.openstreetmap.org/**", (route) => route.abort());
  await page.route("**/api/**", async (route) => {
    const url = new URL(route.request().url());
    const path = url.pathname;
    let body: unknown;

    if (path === "/api/healthcheck/server") {
      body = { axum_version: "test", build_time: NOW, rust_version: "1.test" };
    } else if (path === "/api/healthcheck/state") {
      body = api({
        db_latency: "1ms",
        db_version: "PostgreSQL test",
        responses_handled: 42,
        server_uptime: "1 hour",
        timestamp: NOW,
        users_logged_in: 3,
      });
    } else if (path === "/api/healthcheck/fastfetch") {
      body = api("<span>mobile test host</span>");
    } else if (path === "/api/i18n/ui-text") {
      body = api({ fallback_locale: "en-US", locale: "en-US", texts: {} });
    } else if (path === "/api/auth/me") {
      body = api({
        axum_version: "test",
        build_time: NOW,
        rust_version: "1.test",
        user_info: authMode === "logged-out" ? null : user,
        user_profile_picture:
          authMode === "logged-out"
            ? null
            : { user_profile_picture_link: IMAGE },
      });
    } else if (path === "/api/auth/is-superuser") {
      body = api({ is_superuser: authMode === "superuser" });
    } else if (path === "/api/auth/oidc/status") {
      body = api({ enabled: true, linked: false, provider_name: "Test OIDC" });
    } else if (path === "/api/dropdown/country") {
      body = api({ countries: [] });
    } else if (path === "/api/dropdown/language") {
      body = api([]);
    } else if (path === "/api/user/profile-pictures") {
      body = api({ maximum_profile_pictures: 8, profile_pictures: [] });
    } else if (path.startsWith("/api/users/")) {
      body = api({
        user_country_flag: "🇺🇸",
        user_created_at: NOW,
        user_id: ID,
        user_name: LONG_TEXT,
        user_profile_picture_url: IMAGE,
      });
    } else if (path === "/api/blog/posts" || path === "/api/blog/search") {
      body = api({ available_pages: 1, posts: [postItem] });
    } else if (path.startsWith("/api/blog/")) {
      body = api({
        comments: blogComments,
        post: {
          post_content: `<p>${LONG_TEXT}</p><pre><code>${LONG_TEXT}</code></pre><table><tbody><tr><td>${LONG_TEXT}</td></tr></tbody></table>`,
          post_created_at: NOW,
          post_id: ID,
          post_is_published: true,
          post_metadata: null,
          post_published_at: NOW,
          post_share_count: 2,
          post_slug: "mobile-layout",
          post_summary: LONG_TEXT,
          post_title: LONG_TEXT,
          post_updated_at: NOW,
          post_view_count: 10,
          total_downvotes: 1,
          total_upvotes: 4,
          user_id: ID,
        },
        post_tags: ["responsive"],
        user_badge_info: {
          user_country_flag: "🇺🇸",
          user_name: "mobile-superuser",
          user_profile_picture_url: IMAGE,
        },
        vote_state: 2,
      });
    } else if (path === "/api/forum/capabilities") {
      body = api({
        authenticated: authMode !== "logged-out",
        can_moderate: authMode === "superuser",
        can_post: authMode !== "logged-out",
      });
    } else if (path === "/api/forum/topics") {
      body = api({ next_cursor: null, topics: [forumTopic] });
    } else if (path.startsWith("/api/forum/topics/")) {
      body = api({
        is_subscribed: false,
        next_reply_cursor: null,
        replies: [],
        topic: forumTopic,
      });
    } else if (path === "/api/photographs/get") {
      body = api({
        items: [photograph(ID, LONG_TEXT), photograph(ID_2, "Second photo")],
        pagination: {
          has_next: false,
          has_prev: false,
          page: 1,
          page_size: Number(url.searchParams.get("page_size") ?? 24),
          total_items: 2,
          total_pages: 1,
        },
      });
    } else if (path === "/api/photographs/batches") {
      body = api({ batches: [] });
    } else if (/^\/api\/photographs\/[^/]+$/.test(path)) {
      const id = path.endsWith(ID_2) ? ID_2 : ID;
      body = api({
        comments: photographComments,
        photograph: photograph(id, id === ID_2 ? "Second photo" : LONG_TEXT),
        user_badge_info: {
          user_country_flag: "🇺🇸",
          user_name: "mobile-superuser",
          user_profile_picture_url: IMAGE,
        },
        vote_state: 2,
      });
    } else if (path === "/api/wasm-modules") {
      body = api({
        items: [
          {
            user_id: ID,
            wasm_module_created_at: NOW,
            wasm_module_description: LONG_TEXT,
            wasm_module_id: ID,
            wasm_module_link: "about:blank",
            wasm_module_thumbnail_link: IMAGE,
            wasm_module_title: LONG_TEXT,
            wasm_module_updated_at: NOW,
          },
        ],
      });
    } else if (path === "/api/visitor-board") {
      body = api([]);
    } else if (
      path === "/api/geo-ip-info/me" ||
      path.startsWith("/api/geo-ip-info/")
    ) {
      body = api({
        city: "Denver",
        country_code: "US",
        country_name: "United States",
        ip: "192.0.2.1",
        latitude: 39.7392,
        longitude: -104.9903,
        postal: "80202",
        state: "Colorado",
      });
    } else if (path === "/api/admin/authorization/users") {
      body = api({
        next_cursor: null,
        users: [
          {
            role_id: ID,
            role_name: "admin",
            user_email: user.user_email,
            user_id: ID,
            user_name: user.user_name,
          },
        ],
      });
    } else if (path === "/api/admin/authorization/roles") {
      body = api({
        roles: [{ description: "Administrator", role_id: ID, role_name: "admin" }],
      });
    } else if (path === "/api/admin/authorization/permissions") {
      body = api({
        permissions: [
          {
            description: LONG_TEXT,
            permission_id: ID_2,
            permission_name: "manage.everything",
          },
        ],
      });
    } else if (path === "/api/admin/authorization/role-permissions") {
      body = api({ bindings: [], next_cursor: null });
    } else if (path === "/api/admin/authorization/audit") {
      body = api({
        events: [
          {
            actor_display_name: user.user_name,
            actor_user_id: ID,
            audit_event_id: ID_2,
            created_at: NOW,
            kind: "role_changed",
            new_value: "admin",
            old_value: "user",
            reason: LONG_TEXT,
            request_id: ID,
            role_id: ID,
            role_name: "admin",
            target_display_name: user.user_name,
            target_user_id: ID,
          },
        ],
        next_cursor: null,
      });
    } else if (path === "/api/admin/account-retention-notifications") {
      body = api({
        notifications: [
          {
            attempt_count: 1,
            next_attempt_at: NOW,
            notification_id: ID,
            scheduled_for: NOW,
            stage: "seven_days",
            user_id: ID,
          },
        ],
        next_after_next_attempt_at: null,
        next_after_notification_id: null,
      });
    } else if (path === "/api/admin/media-cleanup/unresolved") {
      body = api({
        records: [
          {
            cleanup_id: ID,
            created_at: NOW,
            original_url: `https://example.test/${LONG_TEXT}`,
            reason: LONG_TEXT,
            source_id: ID_2,
          },
        ],
      });
    } else {
      body = api({});
    }

    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(body),
    });
  });
}

export async function setUiPreferences(
  page: Page,
  locale: "en-US" | "ko-KR",
  theme: "light" | "dark",
): Promise<void> {
  await page.addInitScript(
    ({ locale: nextLocale, theme: nextTheme }) => {
      localStorage.setItem("ui_locale", nextLocale);
      localStorage.setItem("theme", nextTheme);
    },
    { locale, theme },
  );
}
