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
import "../styles/live-chat.css";
import type { UiTextKey } from "../i18n/keys";
import { locale, t, tx } from "../state/i18n";
import { localeDirection } from "../i18n/locales";
import { messageDirection } from "./chat/messageDirection";
import {
  type LiveChatConnectionState,
  useLiveChatSocket,
} from "../state/live_chat_socket";
import { useRtc } from "../state/rtc";
import { UserBadge } from "./UserBadge";
import { CallPanel } from "./call/CallPanel";
import DeleteMessageButton from "./chat/DeleteMessageButton";
import { rememberDeletedMessage } from "../services/live_chat_moderation";
import {
  LIVE_CHAT_HISTORY_LIMIT,
  type WindowUpdate,
  appendLive,
  bufferDetached,
  mergeLatest,
  prependOlder,
  trimOldest,
} from "../services/live_chat_history";
import {
  type LiveChatGreeting,
  greetingFromHello,
} from "../services/live_chat_greeting";

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
const COMPACT_VISIBLE_MESSAGES = 40;
const CONNECTION_LABELS: Readonly<Record<LiveChatConnectionState, UiTextKey>> = {
  connecting: "live_chat.state.connecting",
  open: "live_chat.state.open",
  closed: "live_chat.state.closed",
  error: "live_chat.state.error",
};

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

function containsSentMessage(
  list: readonly LiveChatMessageView[],
  messageId: string,
): boolean {
  return list.some(
    (item) =>
      item.kind === "sent" && item.message.live_chat_message_id === messageId,
  );
}

function oldestSentMessageId(list: readonly LiveChatMessageView[]): string | null {
  for (const item of list) {
    if (item.kind === "sent") return item.message.live_chat_message_id;
  }
  return null;
}

function messageOrderKey(message: LiveChatMessageItem) {
  return { id: message.live_chat_message_id, createdAt: message.message_created_at };
}

/** Unbounded replacement; callers bound the result with `trimOldest`. */
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

  if (replaced) return next;
  return [...next, { kind: "sent" as const, message }];
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
  const deletedMessageIds = new Set<string>();
  const removeMessage = (id: string) => {
    rememberDeletedMessage(deletedMessageIds, id);
    setMessages((current) => current.filter((item) => item.kind !== "sent" || item.message.live_chat_message_id !== id));
  };

  let typingStopTimer: number | undefined;
  let typingRefreshTimer: number | undefined;
  let typingExpiryTimer: number | undefined;
  let typingActive = false;
  let messagesScrollEl: HTMLDivElement | undefined;
  let shouldScrollAfterRender = true;
  let preserveScrollOffsetAfterRender: number | null = null;
  // Set when the window stops reaching the newest message (see
  // services/live_chat_history). Live messages then wait in a bounded buffer
  // until the reader returns to the bottom or sends, and the latest page is
  // merged back in.
  let detached = false;
  let detachedBuffer: LiveChatMessageItem[] = [];
  let resyncing = false;

  const isVisibleMessage = (message: LiveChatMessageItem) =>
    !deletedMessageIds.has(message.live_chat_message_id) && !message.message_deleted_at;

  /** Apply a window update computed inside a setter and follow its side effects. */
  const updateWindow = (
    compute: (list: LiveChatMessageView[]) => WindowUpdate<LiveChatMessageView> | null,
  ): WindowUpdate<LiveChatMessageView> | null => {
    const outcome: { update: WindowUpdate<LiveChatMessageView> | null } = { update: null };
    setMessages((prev) => {
      outcome.update = compute(prev);
      return outcome.update ? outcome.update.list : prev;
    });
    const update = outcome.update;
    if (update?.trimmedOldest) {
      // Dropped history can be paged back in from the new oldest message.
      setNextBeforeMessageId(oldestSentMessageId(update.list));
      setHasMore(true);
    }
    if (update?.detached) detached = true;
    return update;
  };

  const resyncLatest = async () => {
    if (resyncing) return;
    resyncing = true;
    try {
      const response = await liveChatApi.getMessages({ limit: PAGE_SIZE });
      const latest = mergeLatest(
        response.data.items.filter(isVisibleMessage),
        detachedBuffer.filter(isVisibleMessage),
        messageOrderKey,
      );
      detached = false;
      detachedBuffer = [];
      scheduleBottomScroll();
      updateWindow((prev) =>
        trimOldest<LiveChatMessageView>([
          ...latest.map((message) => ({ kind: "sent" as const, message })),
          ...prev.filter((item) => item.kind === "pending"),
        ]),
      );
      setNextBeforeMessageId(latest[0]?.live_chat_message_id ?? null);
      setHasMore(response.data.has_more);
    } catch (err) {
      console.error("Failed to load the latest live chat messages:", err);
    } finally {
      resyncing = false;
    }
  };

  const isFull = () => props.mode === "full";
  const visibleMessages = createMemo(() =>
    isFull() ? messages() : messages().slice(-COMPACT_VISIBLE_MESSAGES),
  );
  // Built once per language: the list can hold many messages and formatters are slow to create.
  const messageTimeFormat = createMemo(
    () =>
      new Intl.DateTimeFormat(locale(), {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      }),
  );
  const typingListFormat = createMemo(
    () => new Intl.ListFormat(locale(), { type: "conjunction", style: "narrow" }),
  );
  const inputCharCount =createMemo(() => Array.from(input()).length);
  const typingText = createMemo(() => {
    const currentKey = actor() ? actorKey(actor()!) : null;
    const names = typingActors()
      .filter((typingActor) => actorKey(typingActor) !== currentKey)
      // Names are user text of unknown direction; isolating each keeps its
      // trailing punctuation and Latin letters from reordering the sentence.
      .map((typingActor) => `\u2068${actorLabel(typingActor)}\u2069`);
    if (names.length === 0) return "";
    if (names.length === 1) return tx("live_chat.typing_one", { name: names[0]! });
    // The list separator is language data: Arabic, Persian, and Urdu join with
    // their own comma and conjunction, not an ASCII ", ".
    return tx("live_chat.typing_many", {
      names: typingListFormat().format(names.slice(0, 2)),
    });
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

      // Scroll only this list: the panel can sit below the fold on the home
      // page, where scrollIntoView would drag the whole document to it.
      if (shouldScrollAfterRender) {
        messagesScrollEl.scrollTop = messagesScrollEl.scrollHeight;
        shouldScrollAfterRender = false;
      }
    });
  };

  // The list itself, not its length: a full window keeps the same length as
  // messages arrive.
  createEffect(visibleMessages, () => {
    restoreOrScrollAfterRender();
  });

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

  const applyGreeting = (greeting: LiveChatGreeting) => {
    setActor(greeting.actor);
    detached = false;
    detachedBuffer = [];
    setMessages(
      greeting.recentMessages.filter(isVisibleMessage).slice(-LIVE_CHAT_HISTORY_LIMIT).map((message) => ({ kind: "sent", message })),
    );
    scheduleBottomScroll();
    setNextBeforeMessageId(greeting.recentMessages[0]?.live_chat_message_id ?? null);
    setHasMore(greeting.hasOlder);
    setConnectedCount(greeting.connectedCount);
  };

  const handleServerEvent = (event: LiveChatServerEvent) => {
    switch (event.type) {
      case "hello":
        applyGreeting(greetingFromHello(event));
        break;
      case "message": {
        const message = event.message;
        if (!isVisibleMessage(message)) break;
        if (detached) {
          detachedBuffer = bufferDetached(detachedBuffer, message);
          break;
        }
        // A scroll still pending means the last message has not rendered yet, so
        // in a burst the measured distance would wrongly say the reader left.
        const atLiveEdge = shouldScrollAfterRender || isNearBottom();
        const update = updateWindow((prev) =>
          containsSentMessage(prev, message.live_chat_message_id)
            ? null
            : appendLive<LiveChatMessageView>(prev, { kind: "sent", message }, atLiveEdge),
        );
        // An unchanged list renders nothing, so nothing would clear the flag.
        if (update) shouldScrollAfterRender = atLiveEdge;
        if (update?.detached) detachedBuffer = bufferDetached(detachedBuffer, message);
        break;
      }
      case "message_ack":
        if (deletedMessageIds.has(event.message.live_chat_message_id) || event.message.message_deleted_at) {
          setMessages((current) => current.filter((item) => item.kind !== "pending" || item.client_message_id !== event.client_message_id));
          break;
        }
        scheduleBottomScroll();
        // The sender is at the live edge, so any overflow drops the oldest.
        updateWindow((prev) =>
          trimOldest(replacePendingMessage(prev, event.client_message_id, event.message)),
        );
        break;
      case "message_deleted":
        removeMessage(event.live_chat_message_id);
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
    // Sending returns the reader to the live edge; a detached window first
    // reloads the latest page, which keeps this pending entry.
    if (detached) void resyncLatest();
    scheduleBottomScroll();
    updateWindow((prev) =>
      trimOldest<LiveChatMessageView>([
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
      ]),
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
      // The reader is at the top: keep the page just loaded and drop the
      // newest entries instead, detaching the window from the live tail.
      updateWindow((prev) =>
        prependOlder<LiveChatMessageView>(
          prev,
          response.data.items
            .filter(isVisibleMessage)
            .map((message) => ({ kind: "sent" as const, message })),
        ),
      );
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
    // A connection the previous route kept open will not be greeted again.
    const greeting = socket.greeting();
    if (greeting) applyGreeting(greeting);
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
      class={[
        pageStyles.card,
        "live-chat-panel flex min-h-0 flex-col",
        isFull() ? "live-chat-panel--full" : "",
      ]}
    >
      <header class="live-chat-panel-header">
        <div class="min-w-0">
          {/* The full page already carries this as its heading. */}
          <Show when={!isFull()}>
            <h2 class="live-chat-title">{t("page.live_chat.title")}</h2>
          </Show>
          <p class="live-chat-status" data-state={connectionState()}>
            {t(CONNECTION_LABELS[connectionState()])} · {connectedCount()}{" "}
            {t("live_chat.online")}
          </p>
        </div>
        <Show when={!isFull()}>
          <div class="flex items-center gap-2">
            <Show when={rtc.participantCount() > 0}>
              <span class={pageStyles.callPill}>
                {tx("live_chat.in_call", { count: rtc.participantCount() })}
              </span>
            </Show>
            <a href="/live-chat" class="live-chat-open">
              {t("live_chat.open")}
            </a>
          </div>
        </Show>
      </header>

      <Show when={isFull()}>
        <CallPanel />
      </Show>

      <div
        ref={messagesScrollEl}
        class="live-chat-messages"
        onScroll={() => {
          if (detached && isNearBottom()) void resyncLatest();
        }}
      >
        <Show when={isFull() && hasMore()}>
          <div class="live-chat-older">
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

        <Show when={visibleMessages().length === 0 && connectionState() === "open"}>
          <p class="live-chat-empty">{t("live_chat.empty")}</p>
        </Show>

        <For each={visibleMessages()}>
          {(message) => (
            <article
              class="live-chat-message"
              data-delivery={message.kind === "pending" ? message.status : "sent"}
            >
              <div class="live-chat-message-head">
                {/* The row follows the page direction so the name and the Delete button keep one order; only the name text is isolated with dir=auto, because the first strong character of the row would be the Delete label or an avatar initial rather than the name. */}
                <span class="live-chat-sender">
                  <Show
                    when={messageUserId(message)}
                    fallback={<bdi dir="auto" class="live-chat-name">{pendingMessageSenderLabel(message)}</bdi>}
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
                  <Show when={message.kind === "sent" ? message.message.live_chat_message_id : null}>
                    {(id) => <DeleteMessageButton messageId={id()} onDeleted={removeMessage} />}
                  </Show>
                </span>
                {/* Month names follow the language, so right-to-left dates must not be forced left to right. */}
                <time class="live-chat-time" dir="auto" datetime={messageCreatedAt(message)}>
                  {messageTimeFormat().format(new Date(messageCreatedAt(message)))}
                </time>
              </div>
              <p class="live-chat-body" dir={messageDirection(messageBody(message))}>{messageBody(message)}</p>
            </article>
          )}
        </For>
      </div>

      <Show when={typingText()}>
        <div class="live-chat-note">{typingText()}</div>
      </Show>

      <Show when={error()}>
        <div class="live-chat-note live-chat-note--error" dir="auto">{error()}</div>
      </Show>

      <form class="live-chat-composer" onSubmit={sendMessage}>
        <div class="live-chat-composer-field">
          <input
            class={pageStyles.input}
            // An empty field has no strong character, and dir=auto then reads as
            // left to right, which would left-align an Arabic placeholder.
            dir={input() ? messageDirection(input()) : localeDirection(locale())}
            maxlength={LIVE_CHAT_MAX_MESSAGE_CHARS}
            value={input()}
            onInput={(event) => handleInput(event.currentTarget.value)}
            placeholder={t("live_chat.message_placeholder")}
            aria-label={t("live_chat.message_placeholder")}
          />
          <div class="live-chat-counter" dir="ltr">
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
