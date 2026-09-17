// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  MinecraftAction,
  MinecraftActionResult,
  MinecraftStatus,
} from "../api-types";
import {
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createMinecraftClient(transport: ApiTransport) {
  return {
    minecraftAction: async (input: {
      readonly body: MinecraftAction;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/admin/minecraft/actions";
      const url = path;
      return requestJson<ApiResponse<MinecraftActionResult>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    minecraftStatus: async (options: ApiRequestOptions = {}) => {
      const path = "/api/admin/minecraft";
      const url = path;
      return requestJson<ApiResponse<MinecraftStatus>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
  } as const;
}
