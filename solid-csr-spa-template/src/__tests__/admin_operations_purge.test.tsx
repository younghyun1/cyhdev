import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import HardPurgePanel from "../components/admin/operations/HardPurgePanel";
import { ADMIN_OPERATION_SECTION_IDS } from "../components/admin/navigation";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import type { AdminOperationsApi } from "../services/contracts/admin_operations";
import { setLocaleSignal, setTexts } from "../state/i18n";

const USER_ID = "019d7f00-0000-7000-8000-000000000020";
const PRIVATE_OBJECT_URL = "https://objects.invalid/private-profile.jpg";

describe("hard-purge operations", () => {
  beforeEach(() => {
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
  });

  afterEach(() => cleanup());

  it("requires the exact bounded phrase and reports partial cleanup safely", async () => {
    const hardPurge = vi.fn().mockResolvedValue(
      response({
        user_id: USER_ID,
        hard_purged_at: "2026-08-30T12:00:00Z",
        profile_objects_deleted: 2,
        profile_metadata_deleted: 2,
        profile_cleanup_remaining: 1,
        profile_cleanup_failures: [
          {
            profile_picture_id: "019d7f00-0000-7000-8000-000000000021",
            object_url: PRIVATE_OBJECT_URL,
            reason: "object store unavailable",
            retryable: true,
          },
        ],
      }),
    );
    const service = { hardPurgeAccount: hardPurge } as Pick<
      AdminOperationsApi,
      "hardPurgeAccount"
    >;

    const result = render(() => <HardPurgePanel service={service} />);
    expect(
      result.container.querySelector(
        `#${ADMIN_OPERATION_SECTION_IDS.hardPurge}`,
      ),
    ).not.toBeNull();
    expect(hardPurge).not.toHaveBeenCalled();

    const idInput = screen.getByLabelText("User UUID") as HTMLInputElement;
    expect(idInput.maxLength).toBe(36);
    idInput.value = USER_ID;
    fireEvent.input(idInput);
    const confirmationInput = await screen.findByLabelText(
      `Type PURGE ${USER_ID} exactly`,
    ) as HTMLInputElement;
    expect(confirmationInput.maxLength).toBe(42);
    confirmationInput.value = `PURGE ${USER_ID}x`;
    fireEvent.input(confirmationInput);
    const checkbox = screen.getByRole("checkbox") as HTMLInputElement;
    checkbox.checked = true;
    fireEvent.change(checkbox);
    const submit = screen.getByRole("button", {
      name: "Hard purge account",
    }) as HTMLButtonElement;
    expect(submit.disabled).toBe(true);

    confirmationInput.value = `PURGE ${USER_ID}`;
    fireEvent.input(confirmationInput);
    await waitFor(() => expect(submit.disabled).toBe(false));
    const form = submit.closest("form");
    if (form === null) throw new Error("expected hard-purge form");
    fireEvent.submit(form);

    await waitFor(() => expect(hardPurge).toHaveBeenCalledWith(USER_ID));
    expect(await screen.findByText(/Identity purge committed/)).toBeTruthy();
    expect(screen.getByText("1")).toBeTruthy();
    expect(screen.getByText("object store unavailable")).toBeTruthy();
    expect(screen.queryByText(PRIVATE_OBJECT_URL)).toBeNull();
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
