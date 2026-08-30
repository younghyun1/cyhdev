// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  DeleteAccountRequest,
  DeleteAccountResponse,
  DeleteProfilePictureResponse,
  HardPurgeAccountResponse,
  IsSuperuserResponse,
  LoginRequest,
  LoginResponse,
  LogoutResponse,
  MeResponse,
  ProfilePictureHistoryResponse,
  PublicUserInfoResponse,
  ResetPasswordProcessRequest,
  ResetPasswordRequest,
  ResetPasswordRequestResponse,
  ResetPasswordResponse,
  ResolveMediaCleanupRequest,
  ResolveMediaCleanupResponse,
  SelectProfilePictureResponse,
  SignupRequest,
  SignupResponse,
  UnresolvedMediaCleanupResponse,
  UpdateProfileRequest,
  UpdateProfileResponse,
  VerifyUserEmailRequest,
  VerifyUserEmailResponse,
} from "../api-types";
import {
  interpolatePath,
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createAccountClient(transport: ApiTransport) {
  return {
    deleteAccount: async (input: {
      readonly body: DeleteAccountRequest;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/auth/account";
      const url = path;
      return requestJson<ApiResponse<DeleteAccountResponse>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    deleteProfilePicture: async (input: {
      readonly path: {
        readonly profile_picture_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/user/profile-pictures/{profile_picture_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<DeleteProfilePictureResponse>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
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
    hardPurgeAccount: async (input: {
      readonly path: {
        readonly user_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/admin/users/{user_id}/hard-purge", input.path);
      const url = path;
      return requestJson<ApiResponse<HardPurgeAccountResponse>>(transport, url, {
        method: "POST",
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
    listProfilePictures: async (options: ApiRequestOptions = {}) => {
      const path = "/api/user/profile-pictures";
      const url = path;
      return requestJson<ApiResponse<ProfilePictureHistoryResponse>>(transport, url, {
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
    resolveMediaCleanup: async (input: {
      readonly body: ResolveMediaCleanupRequest;
      readonly path: {
        readonly cleanup_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/admin/media-cleanup/{cleanup_id}/resolve", input.path);
      const url = path;
      return requestJson<ApiResponse<ResolveMediaCleanupResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    selectProfilePicture: async (input: {
      readonly path: {
        readonly profile_picture_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/user/profile-pictures/{profile_picture_id}/select", input.path);
      const url = path;
      return requestJson<ApiResponse<SelectProfilePictureResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
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
    unresolvedMediaCleanup: async (options: ApiRequestOptions = {}) => {
      const path = "/api/admin/media-cleanup/unresolved";
      const url = path;
      return requestJson<ApiResponse<UnresolvedMediaCleanupResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    updateProfile: async (input: {
      readonly body: UpdateProfileRequest;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/auth/profile";
      const url = path;
      return requestJson<ApiResponse<UpdateProfileResponse>>(transport, url, {
        method: "PATCH",
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
      readonly body: VerifyUserEmailRequest;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/auth/verify-user-email";
      const url = path;
      return requestJson<ApiResponse<VerifyUserEmailResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
  } as const;
}
