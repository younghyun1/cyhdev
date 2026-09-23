// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  DeleteLiveChatMessageResponse,
  GetLiveChatMessagesResponse,
  LiveChatCacheStatsResponse,
} from "../api-types";
import {
  appendQuery,
  interpolatePath,
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createLiveChatClient(transport: ApiTransport) {
  return {
    deleteLiveChatMessage: async (input: {
      readonly path: {
        readonly message_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/admin/live-chat/messages/{message_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<DeleteLiveChatMessageResponse>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    getLiveChatCacheStats: async (options: ApiRequestOptions = {}) => {
      const path = "/api/admin/live-chat/cache-stats";
      const url = path;
      return requestJson<ApiResponse<LiveChatCacheStatsResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    getLiveChatMessages: async (input: {
      readonly query?: {
        readonly before_message_id?: string;
        readonly limit?: number;
      };
    } = {}, options: ApiRequestOptions = {}) => {
      const path = "/api/live-chat/messages";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<GetLiveChatMessagesResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
  } as const;
}
