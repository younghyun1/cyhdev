import { expect, test } from "@playwright/test";
import { installApiMocks, setUiPreferences } from "./fixtures";

const id = "11111111-1111-4111-8111-111111111111";
const message = {
  live_chat_message_id: id, room_key: "main", user_id: null, guest_ip: null,
  sender_kind: 2, sender_display_name: "Guest", sender_country_flag: null,
  user_profile_picture_url: null, message_body: "Message selected for moderation",
  message_created_at: "2026-09-21T12:00:00Z", message_edited_at: null, message_deleted_at: null,
};

for (const mode of ["superuser", "authenticated", "logged-out"] as const) {
  test(`chat deletion respects ${mode} authority and live invalidation`, async ({ page }) => {
    await installApiMocks(page, mode);
    await setUiPreferences(page, "en-US", "light");
    let publish: ((body: string) => void) | undefined;
    await page.routeWebSocket("**/ws/live-chat", (socket) => {
      publish = (body) => socket.send(body);
      socket.send(JSON.stringify({
        type: "hello", actor: { actor_key: { type: "guest", value: "192.0.2.1" },
          sender_kind: 2, user_id: null, guest_ip: null, display_name: "Guest",
          country_flag: null, user_profile_picture_url: null },
        recent_messages: [message], connected_count: 1,
      }));
    });
    let deletes = 0;
    let failDelete = mode === "superuser";
    await page.route(`**/api/admin/live-chat/messages/${id}`, async (route) => {
      expect(route.request().method()).toBe("DELETE");
      deletes++;
      if (failDelete) {
        await route.fulfill({ status: 503, json: { success: false, error: "Unavailable" } });
        return;
      }
      await route.fulfill({ json: { success: true, data: { deleted_message_id: id }, meta: {} } });
    });
    await page.goto("/live-chat");
    await expect(page.getByText(message.message_body)).toBeVisible();
    const button = page.getByRole("button", { name: "Delete", exact: true });
    if (mode === "superuser") {
      const authorGroup = button.locator("..");
      await expect(authorGroup).toContainText("Guest");
      expect(await authorGroup.locator("time").count()).toBe(0);
      const timestamp = page.locator("article time").first();
      const buttonBox = await button.boundingBox();
      const timeBox = await timestamp.boundingBox();
      expect(timeBox!.x).toBeGreaterThan(buttonBox!.x + buttonBox!.width);
      page.once("dialog", (dialog) => dialog.dismiss());
      await button.click();
      expect(deletes).toBe(0);
      await expect(page.getByText(message.message_body)).toBeVisible();
      page.once("dialog", (dialog) => dialog.accept());
      await button.click();
      await expect(page.getByRole("alert")).toBeVisible();
      await expect(page.getByText(message.message_body)).toBeVisible();
      failDelete = false;
      page.once("dialog", (dialog) => dialog.accept());
      await button.click();
      await expect(page.getByText(message.message_body)).toHaveCount(0);
      expect(deletes).toBe(2);
    } else {
      await expect(button).toHaveCount(0);
      expect(publish).toBeDefined();
      publish?.(JSON.stringify({ type: "message_deleted", live_chat_message_id: id }));
      await expect(page.getByText(message.message_body)).toHaveCount(0);
    }
    // A delayed pre-deletion response must not resurrect the removed body.
    publish?.(JSON.stringify({ type: "message", message }));
    await expect(page.getByText(message.message_body)).toHaveCount(0);
  });
}
