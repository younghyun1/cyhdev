// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  OidcAuthorizationResponse,
  OidcLinkCompleteRequest,
  OidcLinkResponse,
  OidcStatusResponse,
  OidcUnlinkRequest,
} from "../api-types";
import {
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createOidcClient(transport: ApiTransport) {
  return {
    completeOidcLink: async (input: {
      readonly body: OidcLinkCompleteRequest;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/auth/oidc/link/complete";
      const url = path;
      return requestJson<ApiResponse<OidcLinkResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    oidcStatus: async (options: ApiRequestOptions = {}) => {
      const path = "/api/auth/oidc/status";
      const url = path;
      return requestJson<ApiResponse<OidcStatusResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    startOidcLink: async (options: ApiRequestOptions = {}) => {
      const path = "/api/auth/oidc/link/start";
      const url = path;
      return requestJson<ApiResponse<OidcAuthorizationResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    startOidcLogin: async (options: ApiRequestOptions = {}) => {
      const path = "/api/auth/oidc/login/start";
      const url = path;
      return requestJson<ApiResponse<OidcAuthorizationResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    unlinkOidc: async (input: {
      readonly body: OidcUnlinkRequest;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/auth/oidc/link";
      const url = path;
      return requestJson<ApiResponse<OidcLinkResponse>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
  } as const;
}
