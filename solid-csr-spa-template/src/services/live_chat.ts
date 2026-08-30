import { API_URL, apiGetJson } from "./api";
import type { ApiResponse } from "../dtos/shared/api_response";
import type { GetLiveChatMessagesRequest } from "../dtos/requests/live_chat";
import type {
  GetLiveChatMessagesResponse,
  LiveChatCacheStatsResponse,
} from "../dtos/responses/live_chat";

function buildQuery(params: GetLiveChatMessagesRequest): string {
  const search = new URLSearchParams();
  if (params.limit !== undefined) {
    search.set("limit", String(params.limit));
  }
  if (params.before_message_id) {
    search.set("before_message_id", params.before_message_id);
  }
  const query = search.toString();
  return query ? `?${query}` : "";
}

export const liveChatApi = {
  getMessages: async (params: GetLiveChatMessagesRequest = {}) =>
    await apiGetJson<ApiResponse<GetLiveChatMessagesResponse>>(
      `/api/live-chat/messages${buildQuery(params)}`,
      { credentials: "include" },
    ),

  getCacheStats: async () =>
    await apiGetJson<ApiResponse<LiveChatCacheStatsResponse>>(
      "/api/live-chat/cache-stats",
      { credentials: "include" },
    ),
};

export function liveChatWebSocketUrl(): string {
  const base = API_URL || window.location.origin;
  const url = new URL("/ws/live-chat", base);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  return url.toString();
}
