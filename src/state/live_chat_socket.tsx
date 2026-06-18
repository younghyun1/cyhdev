import {
  type Accessor,
  type ParentComponent,
  createContext,
  createSignal,
  onCleanup,
  useContext,
} from "solid-js";
import { liveChatWebSocketUrl } from "../services/live_chat";
import {
  LIVE_CHAT_BINARY_PROTOCOL,
  decodeServerEventFrame,
} from "../services/live_chat_binary";
import type { LiveChatServerEvent } from "../dtos/responses/live_chat";

export type LiveChatConnectionState =
  | "connecting"
  | "open"
  | "closed"
  | "error";

type EventHandler = (event: LiveChatServerEvent) => void;

interface LiveChatSocketContextValue {
  connectionState: Accessor<LiveChatConnectionState>;
  /** Whether the negotiated subprotocol is the binary one. */
  isBinary: Accessor<boolean>;
  /** Send a pre-encoded frame. Returns false if the socket is not open. */
  send: (frame: ArrayBuffer | string) => boolean;
  /** Keep the socket connected while held; returns a release function. */
  acquire: () => () => void;
  /** Subscribe to decoded server events; returns an unsubscribe function. */
  onEvent: (handler: EventHandler) => () => void;
}

const RECONNECT_BASE_MS = 1000;
const RECONNECT_MAX_MS = 15000;
// Grace period before closing on the last release, so route transitions that
// briefly drop to zero acquirers reuse the live socket instead of flapping.
const CLOSE_GRACE_MS = 400;

const LiveChatSocketContext = createContext<LiveChatSocketContextValue>();

function parseJsonServerEvent(raw: string): LiveChatServerEvent | null {
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

/**
 * Owns the single `/ws/live-chat` connection shared by chat and RTC signaling.
 * The socket is connected while at least one consumer has `acquire()`d it and
 * closed shortly after the last release. Exactly one connection exists per user.
 */
export const LiveChatSocketProvider: ParentComponent = (props) => {
  const [connectionState, setConnectionState] =
    createSignal<LiveChatConnectionState>("closed");
  const [isBinary, setIsBinary] = createSignal(false);

  let ws: WebSocket | null = null;
  let refCount = 0;
  let disposed = false;
  let reconnectAttempts = 0;
  let reconnectTimer: number | undefined;
  let closeTimer: number | undefined;
  const handlers = new Set<EventHandler>();

  const emit = (event: LiveChatServerEvent) => {
    for (const handler of handlers) {
      try {
        handler(event);
      } catch (err) {
        console.error("live chat event handler error:", err);
      }
    }
  };

  const parseData = (data: unknown): LiveChatServerEvent | null => {
    if (typeof data === "string") return parseJsonServerEvent(data);
    if (data instanceof ArrayBuffer) return decodeServerEventFrame(data);
    return null;
  };

  const clearReconnect = () => {
    if (reconnectTimer !== undefined) {
      window.clearTimeout(reconnectTimer);
      reconnectTimer = undefined;
    }
  };

  const scheduleReconnect = () => {
    if (disposed || refCount <= 0 || reconnectTimer !== undefined) return;
    const delay = Math.min(
      RECONNECT_BASE_MS * 2 ** reconnectAttempts,
      RECONNECT_MAX_MS,
    );
    reconnectAttempts += 1;
    reconnectTimer = window.setTimeout(() => {
      reconnectTimer = undefined;
      connect();
    }, delay);
  };

  const connect = () => {
    if (disposed || refCount <= 0) return;
    if (
      ws &&
      (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)
    ) {
      return;
    }
    setConnectionState("connecting");
    const socket = new WebSocket(liveChatWebSocketUrl(), [
      LIVE_CHAT_BINARY_PROTOCOL,
    ]);
    socket.binaryType = "arraybuffer";
    ws = socket;

    socket.onopen = () => {
      reconnectAttempts = 0;
      setIsBinary(socket.protocol === LIVE_CHAT_BINARY_PROTOCOL);
      setConnectionState("open");
    };
    socket.onclose = () => {
      if (ws === socket) {
        ws = null;
        setConnectionState("closed");
        scheduleReconnect();
      }
    };
    socket.onerror = () => {
      setConnectionState("error");
      // onclose follows and schedules the reconnect.
    };
    socket.onmessage = (event) => {
      const parsed = parseData(event.data);
      if (parsed) emit(parsed);
    };
  };

  const disconnect = () => {
    clearReconnect();
    const socket = ws;
    ws = null;
    if (socket) {
      socket.onclose = null;
      socket.close();
    }
    setConnectionState("closed");
  };

  const clearCloseTimer = () => {
    if (closeTimer !== undefined) {
      window.clearTimeout(closeTimer);
      closeTimer = undefined;
    }
  };

  const acquire = (): (() => void) => {
    clearCloseTimer();
    refCount += 1;
    if (refCount === 1) {
      reconnectAttempts = 0;
      connect();
    }
    let released = false;
    return () => {
      if (released) return;
      released = true;
      refCount = Math.max(0, refCount - 1);
      if (refCount === 0) {
        clearCloseTimer();
        closeTimer = window.setTimeout(() => {
          closeTimer = undefined;
          if (refCount === 0) disconnect();
        }, CLOSE_GRACE_MS);
      }
    };
  };

  const send = (frame: ArrayBuffer | string): boolean => {
    if (!ws || ws.readyState !== WebSocket.OPEN) return false;
    try {
      ws.send(frame);
      return true;
    } catch (err) {
      console.error("live chat send failed:", err);
      return false;
    }
  };

  const onEvent = (handler: EventHandler): (() => void) => {
    handlers.add(handler);
    return () => {
      handlers.delete(handler);
    };
  };

  onCleanup(() => {
    disposed = true;
    clearCloseTimer();
    disconnect();
    handlers.clear();
  });

  const value: LiveChatSocketContextValue = {
    connectionState,
    isBinary,
    send,
    acquire,
    onEvent,
  };

  return (
    <LiveChatSocketContext.Provider value={value}>
      {props.children}
    </LiveChatSocketContext.Provider>
  );
};

export function useLiveChatSocket(): LiveChatSocketContextValue {
  const ctx = useContext(LiveChatSocketContext);
  if (!ctx) {
    throw new Error(
      "useLiveChatSocket must be used within a LiveChatSocketProvider",
    );
  }
  return ctx;
}
