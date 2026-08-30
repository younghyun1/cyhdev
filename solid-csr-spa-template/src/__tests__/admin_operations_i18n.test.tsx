import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { getUiTextBundle } = vi.hoisted(() => ({
  getUiTextBundle: vi.fn(),
}));

vi.mock("../services/all_api", () => ({
  i18nApi: { getUiTextBundle },
}));

import I18nSyncPanel from "../components/admin/operations/I18nSyncPanel";
import { ADMIN_OPERATION_SECTION_IDS } from "../components/admin/navigation";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import type { AdminOperationsApi } from "../services/contracts/admin_operations";
import { setLocaleSignal, setTexts } from "../state/i18n";

describe("UI text synchronization operations", () => {
  beforeEach(() => {
    getUiTextBundle.mockReset();
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
  });

  afterEach(() => cleanup());

  it("does not synchronize on mount and refreshes the active browser bundle", async () => {
    const syncI18n = vi.fn().mockResolvedValue(response({ num_rows: 772 }));
    getUiTextBundle.mockResolvedValue(
      response({
        locale: "en-US",
        texts: { "operations.i18n.title": "Fresh synchronized title" },
      }),
    );
    const service = { syncI18n } as Pick<
      AdminOperationsApi,
      "syncI18n"
    >;

    const result = render(() => <I18nSyncPanel service={service} />);
    expect(
      result.container.querySelector(`#${ADMIN_OPERATION_SECTION_IDS.i18n}`),
    ).not.toBeNull();
    expect(syncI18n).not.toHaveBeenCalled();
    fireEvent.click(
      screen.getByRole("button", { name: "Synchronize UI text" }),
    );

    await waitFor(() => expect(syncI18n).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(getUiTextBundle).toHaveBeenCalledWith("en-US"));
    expect(await screen.findByText("Fresh synchronized title")).toBeTruthy();
    expect(screen.getByText(/loaded 772 locale rows/)).toBeTruthy();
  });

  it("warns that a rejected synchronization can be partially applied", async () => {
    const syncI18n = vi.fn().mockRejectedValue(
      new Error("cache reload failed"),
    );
    const service = { syncI18n } as Pick<
      AdminOperationsApi,
      "syncI18n"
    >;

    render(() => <I18nSyncPanel service={service} />);
    fireEvent.click(
      screen.getByRole("button", { name: "Synchronize UI text" }),
    );

    expect(await screen.findByText("cache reload failed")).toBeTruthy();
    expect(
      screen.getByText(/database rows before a later cache refresh failed/),
    ).toBeTruthy();
    expect(getUiTextBundle).not.toHaveBeenCalled();
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
