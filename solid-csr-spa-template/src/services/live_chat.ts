import { API_URL } from "./api";
import { contractApi } from "./account_api";
import type {
  ApiResponse,
  GetLiveChatMessagesResponse as GeneratedMessagesResponse,
  LiveChatCacheStatsResponse as GeneratedCacheStatsResponse,
  LiveChatMessageItem as GeneratedMessageItem,
} from "../generated";
import type {
  GetLiveChatMessagesResponse,
  LiveChatCacheStatsResponse,
  LiveChatMessageItem,
} from "../dtos/responses/live_chat";

type MessagesInput = NonNullable<
  Parameters<typeof contractApi.getLiveChatMessages>[0]
>;
type MessagesQuery = NonNullable<MessagesInput["query"]>;

function normalizeMessage(message: GeneratedMessageItem): LiveChatMessageItem {
  return {
    live_chat_message_id: message.live_chat_message_id,
    room_key: message.room_key,
    user_id: message.user_id ?? null,
    guest_ip: message.guest_ip ?? null,
    sender_kind: message.sender_kind,
    sender_display_name: message.sender_display_name,
    sender_country_flag: message.sender_country_flag ?? null,
    user_profile_picture_url: message.user_profile_picture_url ?? null,
    message_body: message.message_body,
    message_created_at: message.message_created_at,
    message_edited_at: message.message_edited_at ?? null,
    message_deleted_at: message.message_deleted_at ?? null,
  };
}

function normalizeMessages(
  response: ApiResponse<GeneratedMessagesResponse>,
): ApiResponse<GetLiveChatMessagesResponse> {
  return {
    ...response,
    data: {
      items: response.data.items.map(normalizeMessage),
      next_before_message_id: response.data.next_before_message_id ?? null,
      has_more: response.data.has_more,
    },
  };
}

function normalizeCacheStats(
  response: ApiResponse<GeneratedCacheStatsResponse>,
): ApiResponse<LiveChatCacheStatsResponse> {
  return {
    ...response,
    data: {
      ...response.data,
      oldest_cached_at: response.data.oldest_cached_at ?? null,
      newest_cached_at: response.data.newest_cached_at ?? null,
    },
  };
}

export const liveChatApi = {
  getMessages: async (params: MessagesQuery = {}) =>
    normalizeMessages(await contractApi.getLiveChatMessages({ query: params })),
  getCacheStats: async () =>
    normalizeCacheStats(await contractApi.getLiveChatCacheStats()),
} as const;

export function liveChatWebSocketUrl(): string {
  const base = API_URL || window.location.origin;
  const url = new URL("/ws/live-chat", base);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  return url.toString();
}
