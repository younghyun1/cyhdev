import { describe, expect, it } from "vitest";
import type {
  ChatActor,
  LiveChatMessageItem,
  LiveChatServerEvent,
} from "../dtos/responses/live_chat";
import { LIVE_CHAT_GREETING_MESSAGES, advanceGreeting } from "./live_chat_greeting";

const actor = { display_name: "Guest" } as ChatActor;

const message = (id: number, deleted = false) =>
  ({
    live_chat_message_id: `m${id}`,
    message_deleted_at: deleted ? "2026-11-04T00:00:00Z" : null,
  }) as LiveChatMessageItem;

const hello = (count: number): LiveChatServerEvent => ({
  type: "hello",
  actor,
  recent_messages: Array.from({ length: count }, (_, index) => message(index)),
  connected_count: 3,
});

const ids = (greeting: ReturnType<typeof advanceGreeting>) =>
  greeting?.recentMessages.map((item) => item.live_chat_message_id);

describe("advanceGreeting", () => {
  it("stays empty until the connection is greeted", () => {
    expect(advanceGreeting(null, { type: "presence", connected_count: 4 } as LiveChatServerEvent)).toBeNull();
  });

  it("reports older history only when the greeting was a full page", () => {
    expect(advanceGreeting(null, hello(LIVE_CHAT_GREETING_MESSAGES - 1))?.hasOlder).toBe(false);
    expect(advanceGreeting(null, hello(LIVE_CHAT_GREETING_MESSAGES))?.hasOlder).toBe(true);
  });

  it("follows new, acknowledged, and deleted messages without duplicates", () => {
    let greeting = advanceGreeting(null, hello(2));
    greeting = advanceGreeting(greeting, { type: "message", message: message(2) });
    greeting = advanceGreeting(greeting, { type: "message", message: message(2) });
    greeting = advanceGreeting(greeting, {
      type: "message_ack",
      client_message_id: "c1",
      message: message(3),
    });
    greeting = advanceGreeting(greeting, { type: "message_deleted", live_chat_message_id: "m0" });
    expect(ids(greeting)).toEqual(["m1", "m2", "m3"]);
  });

  it("does not keep a message that arrives already deleted", () => {
    const greeting = advanceGreeting(advanceGreeting(null, hello(1)), {
      type: "message",
      message: message(9, true),
    });
    expect(ids(greeting)).toEqual(["m0"]);
  });

  it("stays bounded, and remembers that trimmed history can be paged back", () => {
    let greeting = advanceGreeting(null, hello(LIVE_CHAT_GREETING_MESSAGES - 1));
    expect(greeting?.hasOlder).toBe(false);
    for (let index = 100; index < 103; index += 1) {
      greeting = advanceGreeting(greeting, { type: "message", message: message(index) });
    }
    expect(greeting?.recentMessages).toHaveLength(LIVE_CHAT_GREETING_MESSAGES);
    expect(greeting?.recentMessages.at(-1)?.live_chat_message_id).toBe("m102");
    expect(greeting?.hasOlder).toBe(true);
  });

  it("tracks presence and ignores events that do not change the greeting", () => {
    const start = advanceGreeting(null, hello(1));
    const next = advanceGreeting(start, { type: "presence", connected_count: 9 } as LiveChatServerEvent);
    expect(next?.connectedCount).toBe(9);
    expect(advanceGreeting(next, { type: "heartbeat_ack" } as LiveChatServerEvent)).toBe(next);
  });

  it("starts over when a new connection greets again", () => {
    const stale = advanceGreeting(advanceGreeting(null, hello(5)), {
      type: "presence",
      connected_count: 9,
    } as LiveChatServerEvent);
    const fresh = advanceGreeting(stale, hello(1));
    expect(fresh?.connectedCount).toBe(3);
    expect(ids(fresh)).toEqual(["m0"]);
  });
});
