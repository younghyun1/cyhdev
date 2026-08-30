// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  CheckIfUserExistsRequest,
  CheckIfUserExistsResponse,
  IsSuperuserResponse,
  LoginRequest,
  LoginResponse,
  LogoutResponse,
  MeResponse,
  PublicUserInfoResponse,
  ResetPasswordProcessRequest,
  ResetPasswordRequest,
  ResetPasswordRequestResponse,
  ResetPasswordResponse,
  SignupRequest,
  SignupResponse,
} from "../api-types";
import {
  appendQuery,
  interpolatePath,
  requestHeaders,
  requestJson,
  requestText,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createAccountClient(transport: ApiTransport) {
  return {
    checkIfUserExists: async (input: {
      readonly body: CheckIfUserExistsRequest;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/auth/check-if-user-exists";
      const url = path;
      return requestJson<ApiResponse<CheckIfUserExistsResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    getUserInfo: async (input: {
      readonly path: {
        readonly user_name: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/users/{user_name}", input.path);
      const url = path;
      return requestJson<ApiResponse<PublicUserInfoResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    isSuperuser: async (options: ApiRequestOptions = {}) => {
      const path = "/api/auth/is-superuser";
      const url = path;
      return requestJson<ApiResponse<IsSuperuserResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    login: async (input: {
      readonly body: LoginRequest;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/auth/login";
      const url = path;
      return requestJson<ApiResponse<LoginResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    logout: async (options: ApiRequestOptions = {}) => {
      const path = "/api/auth/logout";
      const url = path;
      return requestJson<ApiResponse<LogoutResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    me: async (options: ApiRequestOptions = {}) => {
      const path = "/api/auth/me";
      const url = path;
      return requestJson<ApiResponse<MeResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    resetPassword: async (input: {
      readonly body: ResetPasswordProcessRequest;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/auth/reset-password";
      const url = path;
      return requestJson<ApiResponse<ResetPasswordResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    resetPasswordRequest: async (input: {
      readonly body: ResetPasswordRequest;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/auth/reset-password-request";
      const url = path;
      return requestJson<ApiResponse<ResetPasswordRequestResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    signup: async (input: {
      readonly body: SignupRequest;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/auth/signup";
      const url = path;
      return requestJson<ApiResponse<SignupResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    uploadProfilePicture: async (input: {
      readonly body: FormData;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/user/upload-profile-picture";
      const url = path;
      return requestJson<ApiResponse<null>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
        body: input.body,
      });
    },
    verifyUserEmail: async (input: {
      readonly query: {
        readonly email_validation_token_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/auth/verify-user-email";
      const url = appendQuery(path, input.query);
      return requestText<string>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
  } as const;
}
