import type {
  ChatActor,
  LiveChatMessageItem,
  LiveChatServerEvent,
} from "../dtos/responses/live_chat";

/** Messages the server sends in `hello`, and so the most a greeting keeps. */
export const LIVE_CHAT_GREETING_MESSAGES = 50;

/**
 * A connection's `hello`, kept current from the events that followed it.
 *
 * The server greets a connection only once, but the socket outlives route
 * changes. A panel mounting on a connection that is already open starts from
 * this, since no second `hello` is coming.
 */
export interface LiveChatGreeting {
  readonly actor: ChatActor;
  readonly recentMessages: readonly LiveChatMessageItem[];
  readonly connectedCount: number;
  /** History exists before `recentMessages`. */
  readonly hasOlder: boolean;
}

export function greetingFromHello(
  event: Extract<LiveChatServerEvent, { type: "hello" }>,
): LiveChatGreeting {
  return {
    actor: event.actor,
    recentMessages: event.recent_messages,
    connectedCount: event.connected_count,
    hasOlder: event.recent_messages.length >= LIVE_CHAT_GREETING_MESSAGES,
  };
}

function withMessage(
  greeting: LiveChatGreeting,
  message: LiveChatMessageItem,
): LiveChatGreeting {
  if (
    message.message_deleted_at ||
    greeting.recentMessages.some(
      (item) => item.live_chat_message_id === message.live_chat_message_id,
    )
  ) {
    return greeting;
  }
  const appended = [...greeting.recentMessages, message];
  const overflow = appended.length - LIVE_CHAT_GREETING_MESSAGES;
  if (overflow <= 0) return { ...greeting, recentMessages: appended };
  return { ...greeting, recentMessages: appended.slice(overflow), hasOlder: true };
}

/** The greeting after `event`; null until the connection's `hello` arrives. */
export function advanceGreeting(
  current: LiveChatGreeting | null,
  event: LiveChatServerEvent,
): LiveChatGreeting | null {
  if (event.type === "hello") return greetingFromHello(event);
  if (!current) return null;
  switch (event.type) {
    case "message":
    case "message_ack":
      return withMessage(current, event.message);
    case "message_deleted":
      return {
        ...current,
        recentMessages: current.recentMessages.filter(
          (item) => item.live_chat_message_id !== event.live_chat_message_id,
        ),
      };
    case "presence":
      return { ...current, connectedCount: event.connected_count };
    default:
      return current;
  }
}
