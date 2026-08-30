import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import RetentionNotificationsPanel from "../components/admin/operations/RetentionNotificationsPanel";
import { ADMIN_OPERATION_SECTION_IDS } from "../components/admin/navigation";
import type { RetentionNotificationStatusItem } from "../generated";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import type { AdminOperationsApi } from "../services/contracts/admin_operations";
import { setLocaleSignal, setTexts } from "../state/i18n";

const NOTIFICATION_ID = "019d7f00-0000-7000-8000-000000000001";
const USER_ID = "019d7f00-0000-7000-8000-000000000002";

describe("retention notification operations", () => {
  beforeEach(() => {
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
  });

  afterEach(() => cleanup());

  it("does not retry on mount and confirms the exact notification context", async () => {
    const status = vi.fn().mockResolvedValue(
      response({
        notifications: [notification()],
        next_after_next_attempt_at: null,
        next_after_notification_id: null,
      }),
    );
    const retry = vi.fn().mockResolvedValue(
      response({
        notification_id: NOTIFICATION_ID,
        next_attempt_at: "2026-08-30T12:00:00Z",
      }),
    );
    const service = {
      retentionStatus: status,
      retryRetentionNotification: retry,
    } as Pick<
      AdminOperationsApi,
      "retentionStatus" | "retryRetentionNotification"
    >;

    const result = render(() => (
      <RetentionNotificationsPanel service={service} />
    ));

    expect(
      result.container.querySelector(
        `#${ADMIN_OPERATION_SECTION_IDS.retention}`,
      ),
    ).not.toBeNull();
    expect(await screen.findByText("Seven-day warning")).toBeTruthy();
    expect(retry).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Queue retry" }));

    const dialog = await screen.findByRole("dialog");
    expect(dialog.textContent).toContain(NOTIFICATION_ID);
    expect(dialog.textContent).toContain(USER_ID);
    expect(dialog.textContent).toContain("Seven-day warning");
    const checkbox = within(dialog).getByRole("checkbox") as HTMLInputElement;
    checkbox.checked = true;
    fireEvent.change(checkbox);
    const retryButton = within(dialog).getByRole("button", {
      name: "Queue retry",
    }) as HTMLButtonElement;
    await waitFor(() => expect(retryButton.disabled).toBe(false));
    fireEvent.click(retryButton);

    await waitFor(() => expect(retry).toHaveBeenCalledWith(NOTIFICATION_ID));
    await waitFor(() => expect(status).toHaveBeenCalledTimes(2));
  });

  it("replaces pages and sends both keyset cursor fields together", async () => {
    const firstPage = Array.from(
      { length: 25 },
      (_, index) => notification(String(index + 1).padStart(12, "0")),
    );
    const firstNotification = requiredItem(firstPage[0]);
    const lastNotification = requiredItem(firstPage[24]);
    const cursorTime = "2026-09-01T12:00:00Z";
    const cursorId = lastNotification.notification_id;
    const finalNotification = notification("999999999999");
    const status = vi
      .fn()
      .mockResolvedValueOnce(
        response({
          notifications: firstPage,
          next_after_next_attempt_at: cursorTime,
          next_after_notification_id: cursorId,
        }),
      )
      .mockResolvedValueOnce(
        response({
          notifications: [finalNotification],
          next_after_next_attempt_at: null,
          next_after_notification_id: null,
        }),
      );
    const service = {
      retentionStatus: status,
      retryRetentionNotification: vi.fn(),
    } as Pick<
      AdminOperationsApi,
      "retentionStatus" | "retryRetentionNotification"
    >;

    render(() => <RetentionNotificationsPanel service={service} />);
    await screen.findByText(firstNotification.notification_id);
    fireEvent.click(screen.getByRole("button", { name: "Next" }));

    await screen.findByText(finalNotification.notification_id);
    expect(screen.queryByText(firstNotification.notification_id)).toBeNull();
    expect(status.mock.calls[1]?.[0]).toEqual({
      after_next_attempt_at: cursorTime,
      after_notification_id: cursorId,
      limit: 25,
    });
    expect(
      (screen.getByRole("button", { name: "Next" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
  });
});

function notification(
  suffix = "000000000001",
): RetentionNotificationStatusItem {
  return {
    notification_id: `019d7f00-0000-7000-8000-${suffix}`,
    user_id: USER_ID,
    stage: "seven_days_before_purge",
    scheduled_for: "2026-08-30T11:00:00Z",
    next_attempt_at: "2026-08-30T12:00:00Z",
    attempt_count: 1,
    claim_expires_at: null,
    sent_at: null,
    cancelled_at: null,
    last_error: "temporary delivery failure",
  };
}

function response<T>(data: T) {
  return {
    success: true,
    data,
    meta: {
      time_to_process: "1ms",
      timestamp: "2026-08-30T12:00:00Z",
      metadata: null,
    },
  };
}

function requiredItem<T>(value: T | undefined): T {
  if (value === undefined) throw new Error("expected test fixture item");
  return value;
}
