import { API_URL } from "./api";
import { contractApi } from "./account_api";

type MessagesInput = NonNullable<
  Parameters<typeof contractApi.getLiveChatMessages>[0]
>;
type MessagesQuery = NonNullable<MessagesInput["query"]>;

export const liveChatApi = {
  getMessages: (params: MessagesQuery = {}) =>
    contractApi.getLiveChatMessages({ query: params }),
  getCacheStats: () => contractApi.getLiveChatCacheStats(),
} as const;

export function liveChatWebSocketUrl(): string {
  const base = API_URL || window.location.origin;
  const url = new URL("/ws/live-chat", base);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  return url.toString();
}
