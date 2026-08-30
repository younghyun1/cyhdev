// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  SyncI18nCacheResponse,
  UiTextBundleResponse,
} from "../api-types";
import {
  appendQuery,
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createI18nClient(transport: ApiTransport) {
  return {
    getUiTextBundle: async (input: {
      readonly query?: {
        readonly locale?: string | null;
      };
    } = {}, options: ApiRequestOptions = {}) => {
      const path = "/api/i18n/ui-text";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<UiTextBundleResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    syncI18nCache: async (options: ApiRequestOptions = {}) => {
      const path = "/api/admin/sync-i18n-cache";
      const url = path;
      return requestJson<ApiResponse<SyncI18nCacheResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
  } as const;
}
