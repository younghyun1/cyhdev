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
    completeOidcLink: async (
      input: { readonly body: OidcLinkCompleteRequest },
      options: ApiRequestOptions = {},
    ) =>
      requestJson<ApiResponse<OidcLinkResponse>>(
        transport,
        "/api/auth/oidc/link/complete",
        {
          method: "POST",
          headers: requestHeaders(options.headers, true),
          signal: options.signal,
          body: JSON.stringify(input.body),
        },
      ),
    oidcStatus: async (options: ApiRequestOptions = {}) =>
      requestJson<ApiResponse<OidcStatusResponse>>(
        transport,
        "/api/auth/oidc/status",
        {
          method: "GET",
          headers: requestHeaders(options.headers, false),
          signal: options.signal,
        },
      ),
    startOidcLink: async (options: ApiRequestOptions = {}) =>
      requestJson<ApiResponse<OidcAuthorizationResponse>>(
        transport,
        "/api/auth/oidc/link/start",
        {
          method: "POST",
          headers: requestHeaders(options.headers, false),
          signal: options.signal,
        },
      ),
    startOidcLogin: async (options: ApiRequestOptions = {}) =>
      requestJson<ApiResponse<OidcAuthorizationResponse>>(
        transport,
        "/api/auth/oidc/login/start",
        {
          method: "POST",
          headers: requestHeaders(options.headers, false),
          signal: options.signal,
        },
      ),
    unlinkOidc: async (
      input: { readonly body: OidcUnlinkRequest },
      options: ApiRequestOptions = {},
    ) =>
      requestJson<ApiResponse<OidcLinkResponse>>(
        transport,
        "/api/auth/oidc/link",
        {
          method: "DELETE",
          headers: requestHeaders(options.headers, true),
          signal: options.signal,
          body: JSON.stringify(input.body),
        },
      ),
  } as const;
}
