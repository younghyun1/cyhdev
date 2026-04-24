import {
  For,
  Show,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { A } from "@solidjs/router";
import { liveChatApi, liveChatWebSocketUrl } from "../services/live_chat";
import type { LiveChatClientEvent } from "../dtos/requests/live_chat";
import type {
  ChatActor,
  LiveChatMessageItem,
  LiveChatServerEvent,
} from "../dtos/responses/live_chat";
import { pageStyles } from "../styles/pageStyles";

export type LiveChatPanelMode = "compact" | "full";

type ConnectionState = "connecting" | "open" | "closed" | "error";

const PAGE_SIZE = 50;
const TYPING_DEBOUNCE_MS = 900;

function actorKey(actor: ChatActor): string {
  return `${actor.actor_key.type}:${actor.actor_key.value}`;
}

function actorLabel(actor: ChatActor): string {
  if (actor.guest_ip) return actor.guest_ip;
  return actor.display_name;
}

function upsertMessage(
  list: LiveChatMessageItem[],
  message: LiveChatMessageItem,
): LiveChatMessageItem[] {
  if (list.some((item) => item.live_chat_message_id === message.live_chat_message_id)) {
    return list;
  }
  return [...list, message].slice(-300);
}

function parseServerEvent(raw: string): LiveChatServerEvent | null {
  try {
    const parsed = JSON.parse(raw) as LiveChatServerEvent;
    if (typeof parsed === "object" && parsed !== null && "type" in parsed) {
      return parsed;
    }
  } catch (err) {
    console.error("Failed to parse live chat event:", err);
  }
  return null;
}

export default function LiveChatPanel(props: { mode: LiveChatPanelMode }) {
  const [messages, setMessages] = createSignal<LiveChatMessageItem[]>([]);
  const [input, setInput] = createSignal("");
  const [actor, setActor] = createSignal<ChatActor | null>(null);
  const [typingActors, setTypingActors] = createSignal<ChatActor[]>([]);
  const [connectionState, setConnectionState] =
    createSignal<ConnectionState>("connecting");
  const [connectedCount, setConnectedCount] = createSignal(0);
  const [error, setError] = createSignal<string | null>(null);
  const [nextBeforeMessageId, setNextBeforeMessageId] =
    createSignal<string | null>(null);
  const [hasMore, setHasMore] = createSignal(false);
  const [loadingOlder, setLoadingOlder] = createSignal(false);

  let ws: WebSocket | null = null;
  let typingTimer: number | undefined;

  const isFull = () => props.mode === "full";
  const visibleMessages = createMemo(() =>
    isFull() ? messages() : messages().slice(-8),
  );
  const typingText = createMemo(() => {
    const currentKey = actor() ? actorKey(actor()!) : null;
    const names = typingActors()
      .filter((typingActor) => actorKey(typingActor) !== currentKey)
      .map(actorLabel);
    if (names.length === 0) return "";
    if (names.length === 1) return `${names[0]} is typing`;
    return `${names.slice(0, 2).join(", ")} are typing`;
  });

  const connect = () => {
    setConnectionState("connecting");
    setError(null);
    ws = new WebSocket(liveChatWebSocketUrl());

    ws.onopen = () => setConnectionState("open");
    ws.onclose = () => setConnectionState("closed");
    ws.onerror = () => {
      setConnectionState("error");
      setError("Live chat connection failed.");
    };
    ws.onmessage = (event) => {
      if (typeof event.data !== "string") return;
      const serverEvent = parseServerEvent(event.data);
      if (!serverEvent) return;
      handleServerEvent(serverEvent);
    };
  };

  const handleServerEvent = (event: LiveChatServerEvent) => {
    switch (event.type) {
      case "hello":
        setActor(event.actor);
        setMessages(event.recent_messages);
        setNextBeforeMessageId(
          event.recent_messages[0]?.live_chat_message_id ?? null,
        );
        setHasMore(event.recent_messages.length >= PAGE_SIZE);
        break;
      case "message":
        setMessages((prev) => upsertMessage(prev, event.message));
        break;
      case "message_ack":
        setMessages((prev) => upsertMessage(prev, event.message));
        break;
      case "typing":
        setTypingActors((prev) => {
          const key = actorKey(event.actor);
          const withoutActor = prev.filter((item) => actorKey(item) !== key);
          return event.is_typing ? [...withoutActor, event.actor] : withoutActor;
        });
        break;
      case "presence":
        setConnectedCount(event.connected_count);
        break;
      case "heartbeat_ack":
        break;
      case "error":
        if (event.code !== "heartbeat_ack") {
          setError(event.message);
        }
        break;
    }
  };

  const sendEvent = (event: LiveChatClientEvent) => {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
      setError("Live chat is not connected.");
      return;
    }
    ws.send(JSON.stringify(event));
  };

  const sendTyping = (isTyping: boolean) => {
    sendEvent({ type: "typing", is_typing: isTyping });
  };

  const handleInput = (value: string) => {
    setInput(value);
    if (typingTimer !== undefined) {
      window.clearTimeout(typingTimer);
    }
    if (value.trim()) {
      sendTyping(true);
      typingTimer = window.setTimeout(() => sendTyping(false), TYPING_DEBOUNCE_MS);
    } else {
      sendTyping(false);
    }
  };

  const sendMessage = (event: SubmitEvent) => {
    event.preventDefault();
    const body = input().trim();
    if (!body) return;
    sendEvent({
      type: "send_message",
      client_message_id: crypto.randomUUID(),
      body,
    });
    setInput("");
    sendTyping(false);
  };

  const loadOlder = async () => {
    const before = nextBeforeMessageId();
    if (!before || loadingOlder()) return;
    setLoadingOlder(true);
    setError(null);
    try {
      const response = await liveChatApi.getMessages({
        limit: PAGE_SIZE,
        before_message_id: before,
      });
      setMessages((prev) => [...response.data.items, ...prev]);
      setNextBeforeMessageId(response.data.next_before_message_id);
      setHasMore(response.data.has_more);
    } catch (err) {
      console.error("Failed to load older live chat messages:", err);
      setError("Could not load older messages.");
    } finally {
      setLoadingOlder(false);
    }
  };

  onMount(connect);

  onCleanup(() => {
    if (typingTimer !== undefined) {
      window.clearTimeout(typingTimer);
    }
    if (ws && ws.readyState === WebSocket.OPEN) {
      sendTyping(false);
    }
    ws?.close();
  });

  return (
    <section
      class={`${pageStyles.card} flex min-h-0 flex-col ${isFull() ? "h-[calc(100vh-10rem)]" : "h-[28rem]"}`}
    >
      <header class={`${pageStyles.cardHeader} flex items-center justify-between gap-3`}>
        <div>
          <h2 class="text-lg font-semibold">Live Chat</h2>
          <p class={pageStyles.subtitle}>
            {connectionState()} · {connectedCount()} online
          </p>
        </div>
        <Show when={!isFull()}>
          <A href="/live-chat" class={pageStyles.buttonSecondary}>
            Open
          </A>
        </Show>
      </header>

      <div class="min-h-0 flex-1 overflow-y-auto px-4 py-3">
        <Show when={isFull() && hasMore()}>
          <div class="mb-3 flex justify-center">
            <button
              class={pageStyles.buttonGhost}
              type="button"
              disabled={loadingOlder()}
              onClick={loadOlder}
            >
              {loadingOlder() ? "Loading..." : "Load older"}
            </button>
          </div>
        </Show>

        <div class="space-y-3">
          <For each={visibleMessages()}>
            {(message) => (
              <article class="rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-sm dark:border-slate-800 dark:bg-slate-950">
                <div class="mb-1 flex flex-wrap items-center justify-between gap-2 text-xs text-slate-500 dark:text-slate-400">
                  <span class="font-mono font-semibold text-slate-700 dark:text-slate-200">
                    {message.sender_display_name}
                  </span>
                  <time>
                    {new Date(message.message_created_at).toLocaleTimeString([], {
                      hour: "2-digit",
                      minute: "2-digit",
                    })}
                  </time>
                </div>
                <p class="whitespace-pre-wrap break-words text-slate-900 dark:text-slate-100">
                  {message.message_body}
                </p>
                <Show when={message.guest_ip}>
                  <div class="mt-1 font-mono text-[0.7rem] text-amber-700 dark:text-amber-300">
                    guest IP: {message.guest_ip}
                  </div>
                </Show>
              </article>
            )}
          </For>
        </div>
      </div>

      <Show when={typingText()}>
        <div class="px-4 pb-2 text-xs text-slate-500 dark:text-slate-400">
          {typingText()}
        </div>
      </Show>

      <Show when={error()}>
        <div class="mx-4 mb-2 text-xs text-red-600 dark:text-red-300">{error()}</div>
      </Show>

      <form class={`${pageStyles.cardFooter} flex gap-2`} onSubmit={sendMessage}>
        <input
          class={pageStyles.input}
          maxLength={4096}
          value={input()}
          onInput={(event) => handleInput(event.currentTarget.value)}
          placeholder="Message"
        />
        <button
          class={pageStyles.buttonPrimary}
          type="submit"
          disabled={connectionState() !== "open" || !input().trim()}
        >
          Send
        </button>
      </form>
    </section>
  );
}
