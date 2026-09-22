import { describe, expect, it } from "vitest";
import { rememberDeletedMessage } from "./live_chat_moderation";
import { decodeServerEventFrame } from "./live_chat_binary";

describe("live-chat moderation", () => {
  it("decodes deletion and rejects truncated or trailing data", () => {
    const frame = new Uint8Array(17);
    frame[0] = 0x88;
    frame[16] = 1;
    expect(decodeServerEventFrame(frame.buffer)).toEqual({
      type: "message_deleted",
      live_chat_message_id: "00000000-0000-0000-0000-000000000001",
    });
    expect(decodeServerEventFrame(frame.slice(0, 16).buffer)).toBeNull();
    const trailing = new Uint8Array(18);
    trailing.set(frame);
    expect(decodeServerEventFrame(trailing.buffer)).toBeNull();
  });

  it("bounds deletion history and retains the most recently repeated deletion", () => {
    const ids = new Set<string>();
    for (let i = 0; i < 300; i++) rememberDeletedMessage(ids, String(i));
    rememberDeletedMessage(ids, "0");
    rememberDeletedMessage(ids, "300");
    expect(ids.size).toBe(300);
    expect(ids.has("0")).toBe(true);
    expect(ids.has("1")).toBe(false);
    expect(ids.has("300")).toBe(true);
  });
});
