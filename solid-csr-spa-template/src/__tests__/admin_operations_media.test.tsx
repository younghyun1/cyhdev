import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import MediaCleanupPanel from "../components/admin/operations/MediaCleanupPanel";
import type { UnresolvedMediaCleanupItem } from "../generated";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import {
  MEDIA_CLEANUP_BUCKET,
  type AdminOperationsApi,
} from "../services/contracts/admin_operations";
import { setLocaleSignal, setTexts } from "../state/i18n";

const RECORD: UnresolvedMediaCleanupItem = {
  cleanup_id: "019d7f00-0000-7000-8000-000000000010",
  source_id: "019d7f00-0000-7000-8000-000000000011",
  original_url: "https://legacy.invalid/<img src=x onerror=bad()>",
  reason: "legacy address could not be parsed",
  created_at: "2026-08-30T12:00:00Z",
};

describe("media cleanup operations", () => {
  beforeEach(() => {
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
  });

  afterEach(() => cleanup());

  it("requires an explicit exact key and keeps the stored URL inert", async () => {
    const unresolved = vi
      .fn()
      .mockResolvedValueOnce(response({ records: [RECORD] }))
      .mockResolvedValue(response({ records: [] }));
    const resolve = vi.fn().mockResolvedValue(
      response({
        cleanup_id: RECORD.cleanup_id,
        bucket: MEDIA_CLEANUP_BUCKET,
        key: " folder/key ",
        original_url: RECORD.original_url,
      }),
    );
    const service = {
      unresolvedMediaCleanup: unresolved,
      resolveMediaCleanup: resolve,
    } as Pick<
      AdminOperationsApi,
      "unresolvedMediaCleanup" | "resolveMediaCleanup"
    >;

    const result = render(() => <MediaCleanupPanel service={service} />);

    expect(await screen.findByText(RECORD.original_url)).toBeTruthy();
    expect(resolve).not.toHaveBeenCalled();
    expect(result.container.querySelector("img")).toBeNull();
    expect(result.container.querySelector(`a[href="${RECORD.original_url}"]`)).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Reconcile" }));

    const dialog = await screen.findByRole("dialog");
    const bucketInput = within(dialog).getByLabelText(
      "Configured bucket",
    ) as HTMLInputElement;
    expect(bucketInput.value).toBe(MEDIA_CLEANUP_BUCKET);
    expect(bucketInput.readOnly).toBe(true);
    const keyInput = within(dialog).getByLabelText(
      "Exact object key",
    ) as HTMLInputElement;
    keyInput.value = " folder/key ";
    fireEvent.input(keyInput);
    const checkbox = within(dialog).getByRole("checkbox") as HTMLInputElement;
    checkbox.checked = true;
    fireEvent.change(checkbox);
    const submit = within(dialog).getByRole("button", {
      name: "Reconcile",
    }) as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    const form = dialog.querySelector("form");
    if (form === null) throw new Error("expected reconciliation form");
    fireEvent.submit(form);

    await waitFor(() => {
      expect(resolve).toHaveBeenCalledWith(RECORD.cleanup_id, {
        expected_original_url: RECORD.original_url,
        bucket: MEDIA_CLEANUP_BUCKET,
        key: " folder/key ",
      });
    });
    await waitFor(() => expect(unresolved).toHaveBeenCalledTimes(2));
    expect(await screen.findByText(new RegExp(RECORD.cleanup_id))).toBeTruthy();
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
