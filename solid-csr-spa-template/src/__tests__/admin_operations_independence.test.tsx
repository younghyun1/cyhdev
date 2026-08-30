import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import MediaCleanupPanel from "../components/admin/operations/MediaCleanupPanel";
import RetentionNotificationsPanel from "../components/admin/operations/RetentionNotificationsPanel";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import type { AdminOperationsApi } from "../services/contracts/admin_operations";
import { setLocaleSignal, setTexts } from "../state/i18n";

describe("administration panel state isolation", () => {
  beforeEach(() => {
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
  });

  afterEach(() => cleanup());

  it("keeps another panel usable when one panel load fails", async () => {
    const retentionService = {
      retentionStatus: vi.fn().mockRejectedValue(new Error("retention unavailable")),
      retryRetentionNotification: vi.fn(),
    } as Pick<
      AdminOperationsApi,
      "retentionStatus" | "retryRetentionNotification"
    >;
    const mediaService = {
      unresolvedMediaCleanup: vi.fn().mockResolvedValue(
        response({
          records: [
            {
              cleanup_id: "019d7f00-0000-7000-8000-000000000030",
              source_id: "019d7f00-0000-7000-8000-000000000031",
              original_url: "https://legacy.invalid/profile.jpg",
              reason: "legacy address",
              created_at: "2026-08-30T12:00:00Z",
            },
          ],
        }),
      ),
      resolveMediaCleanup: vi.fn(),
    } as Pick<
      AdminOperationsApi,
      "unresolvedMediaCleanup" | "resolveMediaCleanup"
    >;

    render(() => (
      <>
        <RetentionNotificationsPanel service={retentionService} />
        <MediaCleanupPanel service={mediaService} />
      </>
    ));

    expect(await screen.findByText("retention unavailable")).toBeTruthy();
    expect(
      await screen.findByText("https://legacy.invalid/profile.jpg"),
    ).toBeTruthy();
  });
});

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
