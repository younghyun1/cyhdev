import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onSettled,
} from "solid-js";
import { liveChatApi } from "../services/live_chat";
import {
  encodePingFrame,
  encodeSendMessageFrame,
  encodeTypingFrame,
} from "../services/live_chat_binary";
import type { LiveChatClientEvent } from "../dtos/requests/live_chat";
import type {
  ChatActor,
  LiveChatMessageItem,
  LiveChatServerEvent,
} from "../dtos/responses/live_chat";
import { pageStyles } from "../styles/pageStyles";
import { t, tx } from "../state/i18n";
import { useLiveChatSocket } from "../state/live_chat_socket";
import { useRtc } from "../state/rtc";
import { UserBadge } from "./UserBadge";
import { CallPanel } from "./call/CallPanel";

export type LiveChatPanelMode = "compact" | "full";

type PendingMessageStatus = "sending" | "failed";

type LiveChatMessageView =
  | {
      kind: "sent";
      message: LiveChatMessageItem;
    }
  | {
      kind: "pending";
      client_message_id: string;
      user_id: string | null;
      sender_display_name: string;
      sender_country_flag: string | null;
      user_profile_picture_url: string | null;
      message_body: string;
      message_created_at: string;
      status: PendingMessageStatus;
    };

const PAGE_SIZE = 50;
const TYPING_IDLE_STOP_MS = 1500;
const TYPING_REFRESH_MS = 2000;
const LIVE_CHAT_MAX_MESSAGE_CHARS = 300;
const AUTO_SCROLL_BOTTOM_THRESHOLD_PX = 80;

function actorKey(actor: ChatActor): string {
  return `${actor.actor_key.type}:${actor.actor_key.value}`;
}

function actorLabel(actor: ChatActor): string {
  return appendFlag(actor.display_name, actor.country_flag);
}

function appendFlag(label: string, countryFlag: string | null): string {
  return countryFlag ? `${label} ${countryFlag}` : label;
}

function messageSenderLabel(message: LiveChatMessageItem): string {
  return appendFlag(message.sender_display_name, message.sender_country_flag);
}

function pendingMessageSenderLabel(message: LiveChatMessageView): string {
  if (message.kind === "sent") return messageSenderLabel(message.message);
  return appendFlag(message.sender_display_name, message.sender_country_flag);
}

function messageUserId(message: LiveChatMessageView): string | null {
  if (message.kind === "sent") return message.message.user_id;
  return message.user_id;
}

function messageSenderName(message: LiveChatMessageView): string {
  if (message.kind === "sent") return message.message.sender_display_name;
  return message.sender_display_name;
}

function messageSenderCountryFlag(message: LiveChatMessageView): string | null {
  if (message.kind === "sent") return message.message.sender_country_flag;
  return message.sender_country_flag;
}

function messageSenderProfilePictureUrl(
  message: LiveChatMessageView,
): string | null {
  if (message.kind === "sent") return message.message.user_profile_picture_url;
  return message.user_profile_picture_url;
}

function messageCreatedAt(message: LiveChatMessageView): string {
  if (message.kind === "sent") return message.message.message_created_at;
  return message.message_created_at;
}

function messageBody(message: LiveChatMessageView): string {
  if (message.kind === "sent") return message.message.message_body;
  return message.message_body;
}

function limitMessageInput(value: string): string {
  const chars = Array.from(value);
  if (chars.length <= LIVE_CHAT_MAX_MESSAGE_CHARS) return value;
  return chars.slice(0, LIVE_CHAT_MAX_MESSAGE_CHARS).join("");
}

function upsertMessage(
  list: LiveChatMessageView[],
  message: LiveChatMessageItem,
): LiveChatMessageView[] {
  if (
    list.some(
      (item) =>
        item.kind === "sent" &&
        item.message.live_chat_message_id === message.live_chat_message_id,
    )
  ) {
    return list;
  }
  return [...list, { kind: "sent" as const, message }].slice(-300);
}

function replacePendingMessage(
  list: LiveChatMessageView[],
  clientMessageId: string,
  message: LiveChatMessageItem,
): LiveChatMessageView[] {
  let replaced = false;
  const withoutDuplicateSent = list.filter(
    (item) =>
      item.kind !== "sent" ||
      item.message.live_chat_message_id !== message.live_chat_message_id,
  );
  const next = withoutDuplicateSent.map((item) => {
    if (item.kind === "pending" && item.client_message_id === clientMessageId) {
      replaced = true;
      return { kind: "sent" as const, message };
    }
    return item;
  });

  if (replaced) return next.slice(-300);
  return [...next, { kind: "sent" as const, message }].slice(-300);
}

function markPendingMessagesFailed(
  list: LiveChatMessageView[],
): LiveChatMessageView[] {
  return list.map((item) => {
    if (item.kind === "pending" && item.status === "sending") {
      return { ...item, status: "failed" };
    }
    return item;
  });
}

export default function LiveChatPanel(props: { mode: LiveChatPanelMode }) {
  const socket = useLiveChatSocket();
  const rtc = useRtc();

  const [messages, setMessages] = createSignal<LiveChatMessageView[]>([]);
  const [input, setInput] = createSignal("");
  const [actor, setActor] = createSignal<ChatActor | null>(null);
  const [typingActors, setTypingActors] = createSignal<ChatActor[]>([]);
  const [connectedCount, setConnectedCount] = createSignal(0);
  const [error, setError] = createSignal<string | null>(null);
  const [nextBeforeMessageId, setNextBeforeMessageId] = createSignal<
    string | null
  >(null);
  const [hasMore, setHasMore] = createSignal(false);
  const [loadingOlder, setLoadingOlder] = createSignal(false);

  const connectionState = socket.connectionState;

  let typingStopTimer: number | undefined;
  let typingRefreshTimer: number | undefined;
  let typingExpiryTimer: number | undefined;
  let typingActive = false;
  let messagesScrollEl: HTMLDivElement | undefined;
  let messagesEndEl: HTMLDivElement | undefined;
  let shouldScrollAfterRender = true;
  let preserveScrollOffsetAfterRender: number | null = null;

  const isFull = () => props.mode === "full";
  const visibleMessages = createMemo(() =>
    isFull() ? messages() : messages().slice(-8),
  );
  const inputCharCount = createMemo(() => Array.from(input()).length);
  const typingText = createMemo(() => {
    const currentKey = actor() ? actorKey(actor()!) : null;
    const names = typingActors()
      .filter((typingActor) => actorKey(typingActor) !== currentKey)
      .map(actorLabel);
    if (names.length === 0) return "";
    if (names.length === 1) return tx("live_chat.typing_one", { name: names[0]! });
    return tx("live_chat.typing_many", { names: names.slice(0, 2).join(", ") });
  });

  const scheduleTypingExpiry = (expiresAt: string) => {
    if (typingExpiryTimer !== undefined) {
      window.clearTimeout(typingExpiryTimer);
      typingExpiryTimer = undefined;
    }

    const delay = new Date(expiresAt).getTime() - Date.now() + 100;
    typingExpiryTimer = window.setTimeout(
      () => setTypingActors([]),
      Math.max(delay, 0),
    );
  };

  const isNearBottom = () => {
    if (!messagesScrollEl) return true;
    const remaining =
      messagesScrollEl.scrollHeight -
      messagesScrollEl.scrollTop -
      messagesScrollEl.clientHeight;
    return remaining <= AUTO_SCROLL_BOTTOM_THRESHOLD_PX;
  };

  const scheduleBottomScroll = () => {
    shouldScrollAfterRender = true;
  };

  const restoreOrScrollAfterRender = () => {
    window.requestAnimationFrame(() => {
      if (!messagesScrollEl) return;
      if (preserveScrollOffsetAfterRender !== null) {
        messagesScrollEl.scrollTop =
          messagesScrollEl.scrollHeight - preserveScrollOffsetAfterRender;
        preserveScrollOffsetAfterRender = null;
        return;
      }

      if (shouldScrollAfterRender) {
        messagesEndEl?.scrollIntoView({ block: "end" });
        shouldScrollAfterRender = false;
      }
    });
  };

  createEffect(
    () => visibleMessages().length,
    () => {
      restoreOrScrollAfterRender();
    },
  );

  // Surface transport errors from the shared socket and clear them on recovery.
  createEffect(
    () => connectionState(),
    (state) => {
      if (state === "error") {
        setError(t("live_chat.connection_failed"));
      } else if (state === "open") {
        setError(null);
      }
    },
  );

  const handleServerEvent = (event: LiveChatServerEvent) => {
    switch (event.type) {
      case "hello":
        setActor(event.actor);
        setMessages(
          event.recent_messages.map((message) => ({ kind: "sent", message })),
        );
        scheduleBottomScroll();
        setNextBeforeMessageId(
          event.recent_messages[0]?.live_chat_message_id ?? null,
        );
        setHasMore(event.recent_messages.length >= PAGE_SIZE);
        setConnectedCount(event.connected_count);
        break;
      case "message":
        shouldScrollAfterRender = isNearBottom();
        setMessages((prev) => upsertMessage(prev, event.message));
        break;
      case "message_ack":
        scheduleBottomScroll();
        setMessages((prev) =>
          replacePendingMessage(prev, event.client_message_id, event.message),
        );
        break;
      case "typing":
        setTypingActors((prev) => {
          const key = actorKey(event.actor);
          const withoutActor = prev.filter((item) => actorKey(item) !== key);
          return event.is_typing ? [...withoutActor, event.actor] : withoutActor;
        });
        if (event.is_typing) {
          scheduleTypingExpiry(event.expires_at);
        }
        break;
      case "typing_set":
        setTypingActors(event.actors);
        if (event.actors.length > 0) {
          scheduleTypingExpiry(event.expires_at);
        } else if (typingExpiryTimer !== undefined) {
          window.clearTimeout(typingExpiryTimer);
          typingExpiryTimer = undefined;
        }
        break;
      case "presence":
        setConnectedCount(event.connected_count);
        break;
      case "heartbeat_ack":
        break;
      case "error":
        if (event.code !== "heartbeat_ack") {
          setError(event.message);
          setMessages(markPendingMessagesFailed);
        }
        break;
      case "rtc":
        // Handled by the RTC context; ignored here.
        break;
    }
  };

  const sendEvent = (event: LiveChatClientEvent): boolean => {
    if (connectionState() !== "open") {
      setError(t("live_chat.not_connected"));
      return false;
    }
    try {
      if (socket.isBinary() && event.type !== "rtc") {
        switch (event.type) {
          case "send_message":
            return socket.send(
              encodeSendMessageFrame(event.client_message_id, event.body),
            );
          case "typing":
            return socket.send(encodeTypingFrame(event.is_typing));
          case "heartbeat":
            return socket.send(encodePingFrame(parseHeartbeatNonce(event.nonce)));
        }
      }
      return socket.send(JSON.stringify(event));
    } catch (err) {
      console.error("Failed to send live chat event:", err);
      setError(t("live_chat.send_failed"));
      return false;
    }
  };

  const sendTyping = (isTyping: boolean) => {
    sendEvent({ type: "typing", is_typing: isTyping });
  };

  const parseHeartbeatNonce = (nonce: string): bigint => {
    try {
      return BigInt(nonce);
    } catch {
      return 0n;
    }
  };

  const clearTypingTimers = () => {
    if (typingStopTimer !== undefined) {
      window.clearTimeout(typingStopTimer);
      typingStopTimer = undefined;
    }
    if (typingRefreshTimer !== undefined) {
      window.clearInterval(typingRefreshTimer);
      typingRefreshTimer = undefined;
    }
  };

  const stopTyping = () => {
    if (!typingActive) {
      clearTypingTimers();
      return;
    }
    typingActive = false;
    clearTypingTimers();
    sendTyping(false);
  };

  const startTyping = () => {
    if (!typingActive) {
      typingActive = true;
      sendTyping(true);
      typingRefreshTimer = window.setInterval(
        () => sendTyping(true),
        TYPING_REFRESH_MS,
      );
    }

    if (typingStopTimer !== undefined) {
      window.clearTimeout(typingStopTimer);
    }
    typingStopTimer = window.setTimeout(stopTyping, TYPING_IDLE_STOP_MS);
  };

  const handleInput = (value: string) => {
    const limitedValue = limitMessageInput(value);
    setInput(limitedValue);
    if (limitedValue.trim()) {
      startTyping();
    } else {
      stopTyping();
    }
  };

  const sendMessage = (event: SubmitEvent) => {
    event.preventDefault();
    const body = input().trim();
    if (!body) return;
    if (Array.from(body).length > LIVE_CHAT_MAX_MESSAGE_CHARS) {
      setError(
        tx("live_chat.message_too_long", { count: LIVE_CHAT_MAX_MESSAGE_CHARS }),
      );
      return;
    }
    const clientMessageId = crypto.randomUUID();
    const sent = sendEvent({
      type: "send_message",
      client_message_id: clientMessageId,
      body,
    });
    if (!sent) return;

    const currentActor = actor();
    scheduleBottomScroll();
    setMessages((prev) =>
      [
        ...prev,
        {
          kind: "pending" as const,
          client_message_id: clientMessageId,
          user_id: currentActor?.user_id ?? null,
          sender_display_name: currentActor?.display_name ?? t("live_chat.you"),
          sender_country_flag: currentActor?.country_flag ?? null,
          user_profile_picture_url:
            currentActor?.user_profile_picture_url ?? null,
          message_body: body,
          message_created_at: new Date().toISOString(),
          status: "sending" as const,
        },
      ].slice(-300),
    );
    setInput("");
    stopTyping();
  };

  const loadOlder = async () => {
    const before = nextBeforeMessageId();
    if (!before || loadingOlder()) return;
    if (messagesScrollEl) {
      preserveScrollOffsetAfterRender =
        messagesScrollEl.scrollHeight - messagesScrollEl.scrollTop;
    }
    setLoadingOlder(true);
    setError(null);
    try {
      const response = await liveChatApi.getMessages({
        limit: PAGE_SIZE,
        before_message_id: before,
      });
      setMessages((prev) => [
        ...response.data.items.map((message) => ({
          kind: "sent" as const,
          message,
        })),
        ...prev,
      ]);
      setNextBeforeMessageId(response.data.next_before_message_id);
      setHasMore(response.data.has_more);
    } catch (err) {
      console.error("Failed to load older live chat messages:", err);
      setError(t("live_chat.load_older_failed"));
    } finally {
      setLoadingOlder(false);
    }
  };

  onSettled(() => {
    const release = socket.acquire();
    const unsubscribe = socket.onEvent(handleServerEvent);
    return () => {
      unsubscribe();
      release();
    };
  });

  onCleanup(() => {
    stopTyping();
    if (typingExpiryTimer !== undefined) {
      window.clearTimeout(typingExpiryTimer);
    }
  });

  return (
    <section
      class={`${pageStyles.card} flex min-h-0 flex-col ${isFull() ? "h-[calc(100vh-12rem)] max-h-[44rem]" : "h-[28rem] max-h-[calc(100vh-10rem)]"}`}
    >
      <header
        class={`${pageStyles.cardHeader} flex items-center justify-between gap-3`}
      >
        <div>
          <h2 class="text-lg font-semibold">{t("page.live_chat.title")}</h2>
          <p class={pageStyles.subtitle}>
            {connectionState()} · {connectedCount()} {t("live_chat.online")}
          </p>
        </div>
        <div class="flex items-center gap-2">
          <Show when={!isFull() && rtc.participantCount() > 0}>
            <span class={pageStyles.callPill}>
              {rtc.participantCount()} in call
            </span>
          </Show>
          <Show when={!isFull()}>
            <a href="/live-chat" class={pageStyles.buttonSecondary}>
              {t("live_chat.open")}
            </a>
          </Show>
        </div>
      </header>

      <Show when={isFull()}>
        <CallPanel />
      </Show>

      <div
        ref={messagesScrollEl}
        class="min-h-0 flex-1 overflow-y-auto px-4 py-3"
      >
        <Show when={isFull() && hasMore()}>
          <div class="mb-3 flex justify-center">
            <button
              class={pageStyles.buttonGhost}
              type="button"
              disabled={loadingOlder()}
              onClick={loadOlder}
            >
              {loadingOlder() ? t("live_chat.loading") : t("live_chat.load_older")}
            </button>
          </div>
        </Show>

        <div class="space-y-3">
          <For each={visibleMessages()}>
            {(message) => (
              <article
                class={`rounded-sm border px-3 py-2 text-sm ${
                  message.kind === "pending"
                    ? message.status === "failed"
                      ? "border-danger/30 bg-danger/10 text-danger"
                      : "border-line bg-surface-2 text-ink-muted opacity-70"
                    : "border-line bg-surface"
                }`}
              >
                <div class="mb-1 flex flex-wrap items-center justify-between gap-2 text-xs text-ink-muted">
                  <span
                    class={`font-mono font-semibold ${
                      message.kind === "pending"
                        ? ""
                        : "text-ink"
                    }`}
                  >
                    <Show
                      when={messageUserId(message)}
                      fallback={pendingMessageSenderLabel(message)}
                    >
                      <UserBadge
                        userName={messageSenderName(message)}
                        profilePictureUrl={
                          messageSenderProfilePictureUrl(message) ?? undefined
                        }
                        countryFlag={messageSenderCountryFlag(message)}
                        size="sm"
                      />
                    </Show>
                  </span>
                  <time>
                    {new Date(messageCreatedAt(message)).toLocaleTimeString([], {
                      hour: "2-digit",
                      minute: "2-digit",
                    })}
                  </time>
                </div>
                <p
                  class={`whitespace-pre-wrap break-words ${
                    message.kind === "pending"
                      ? ""
                      : "text-ink"
                  }`}
                >
                  {messageBody(message)}
                </p>
              </article>
            )}
          </For>
          <div ref={messagesEndEl} aria-hidden="true" />
        </div>
      </div>

      <Show when={typingText()}>
        <div class="px-4 pb-2 text-xs text-ink-muted">
          {typingText()}
        </div>
      </Show>

      <Show when={error()}>
        <div class="mx-4 mb-2 text-xs text-danger">
          {error()}
        </div>
      </Show>

      <form
        class={`${pageStyles.cardFooter} flex items-start gap-2`}
        onSubmit={sendMessage}
      >
        <div class="min-w-0 flex-1">
          <input
            class={pageStyles.input}
            maxlength={LIVE_CHAT_MAX_MESSAGE_CHARS}
            value={input()}
            onInput={(event) => handleInput(event.currentTarget.value)}
            placeholder={t("live_chat.message_placeholder")}
          />
          <div class="mt-1 text-right font-mono text-[0.65rem] text-ink-muted">
            {inputCharCount()}/{LIVE_CHAT_MAX_MESSAGE_CHARS}
          </div>
        </div>
        <button
          class={pageStyles.buttonPrimary}
          type="submit"
          disabled={connectionState() !== "open" || !input().trim()}
        >
          {t("live_chat.send")}
        </button>
      </form>
    </section>
  );
}
