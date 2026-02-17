import { apiFetch } from "./api";
import axios, { AxiosRequestConfig } from "axios";

// Helper to interpolate path params, e.g. /path/{id}
function interpolate(
  path: string,
  params: Record<string, string | number | boolean | undefined> = {},
) {
  return path.replace(/{([^}]+)}/g, (_, key) =>
    encodeURIComponent(String(params[key] ?? "")),
  );
}

type RequestOptions = Omit<RequestInit, "body"> & {
  body?: object;
  params?: Record<string, string | number | boolean | undefined>;
};

type HttpError = Error & {
  status: number;
  body_text: string;
};

function buildHttpError(status: number, bodyText: string): HttpError {
  let message = `Request failed (${status})`;
  if (bodyText) {
    try {
      const parsed = JSON.parse(bodyText) as { message?: unknown };
      if (typeof parsed.message === "string" && parsed.message.trim()) {
        message = parsed.message.trim();
      } else {
        message = bodyText;
      }
    } catch {
      message = bodyText;
    }
  }

  const err = new Error(message) as HttpError;
  err.status = status;
  err.body_text = bodyText;
  return err;
}

/**
 * Generic HTTP GET
 */
async function get<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const url = interpolate(path, options.params);
  const { body: _, params: __, ...fetchOptions } = options;
  const res = await apiFetch(url, { ...fetchOptions, method: "GET" });
  if (!res.ok) {
    const bodyText = await res.text();
    throw buildHttpError(res.status, bodyText);
  }
  return res.json();
}

/**
 * Generic HTTP POST
 */
async function post<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const url = interpolate(path, options.params);
  const { body, params: _, ...restOptions } = options;
  const fetchOpts: RequestInit = {
    ...restOptions,
    method: "POST",
    body: body !== undefined ? JSON.stringify(body) : undefined,
    headers: {
      "Content-Type": "application/json",
      ...options.headers,
    },
  };
  const res = await apiFetch(url, fetchOpts);
  if (!res.ok) {
    const bodyText = await res.text();
    throw buildHttpError(res.status, bodyText);
  }
  // Handle 204/empty-body responses gracefully for side-effect endpoints (e.g., vote rescinds)
  if (res.status === 204) {
    return undefined as unknown as T;
  }
  const contentLength = res.headers.get("content-length");
  if (contentLength === "0") {
    return undefined as unknown as T;
  }
  try {
    return await res.json();
  } catch {
    return undefined as unknown as T;
  }
}

/**
 * Generic HTTP POST for FormData
 * Uses Axios so we can get real upload progress events.
 */
async function postFormData<T>(
  path: string,
  formData: FormData,
  options: Omit<RequestOptions, "body" | "params"> & {
    onUploadProgress?: (percent: number) => void;
    params?: Record<string, string | number | undefined>;
  } = {},
): Promise<T> {
  const url = interpolate(path, options.params);

  const config: AxiosRequestConfig<FormData> = {
    url,
    method: "POST",
    data: formData,
    // Normalize headers to a plain Record<string, string> so TypeScript is happy
    headers: {
      ...(options.headers
        ? Object.fromEntries(
            Object.entries(options.headers as Record<string, string>),
          )
        : {}),
      // Let Axios set the correct multipart boundary. If you prefer, you can omit this
      // header entirely and Axios will infer it from the FormData.
      "Content-Type": "multipart/form-data",
    },
    withCredentials:
      typeof options.credentials !== "undefined"
        ? options.credentials === "include"
        : true,
    onUploadProgress: (event) => {
      if (!options.onUploadProgress || !event.total) return;
      const percent = Math.round((event.loaded * 100) / event.total);
      options.onUploadProgress(percent);
    },
  };

  const res = await axios.request<T>(config);
  return res.data;
}

/**
 * Generic HTTP PATCH
 */
async function patch<T>(
  path: string,
  options: RequestOptions = {},
): Promise<T> {
  const url = interpolate(path, options.params);
  const { body, params: _, ...restOptions } = options;
  const fetchOpts: RequestInit = {
    ...restOptions,
    method: "PATCH",
    body: body !== undefined ? JSON.stringify(body) : undefined,
    headers: {
      "Content-Type": "application/json",
      ...options.headers,
    },
  };
  const res = await apiFetch(url, fetchOpts);
  if (!res.ok) {
    const bodyText = await res.text();
    throw buildHttpError(res.status, bodyText);
  }
  // Handle 204/empty-body responses gracefully for side-effect endpoints (e.g., vote rescinds)
  if (res.status === 204) {
    return undefined as unknown as T;
  }
  const contentLength = res.headers.get("content-length");
  if (contentLength === "0") {
    return undefined as unknown as T;
  }
  try {
    return await res.json();
  } catch {
    return undefined as unknown as T;
  }
}

/**
 * Generic HTTP DELETE
 */
async function del<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const url = interpolate(path, options.params);
  const { body, params: _, ...restOptions } = options;
  const fetchOpts: RequestInit = {
    ...restOptions,
    method: "DELETE",
    body: body !== undefined ? JSON.stringify(body) : undefined,
    headers:
      body !== undefined
        ? { "Content-Type": "application/json", ...options.headers }
        : { ...options.headers },
  };
  const res = await apiFetch(url, fetchOpts);
  if (!res.ok) {
    const bodyText = await res.text();
    throw buildHttpError(res.status, bodyText);
  }
  if (res.status === 204) {
    return undefined as unknown as T;
  }
  const contentLength = res.headers.get("content-length");
  if (contentLength === "0") {
    return undefined as unknown as T;
  }
  try {
    return await res.json();
  } catch {
    return undefined as unknown as T;
  }
}

// Import all DTO types
import type { ApiResponse } from "../dtos/shared/api_response";

import type {
  HealthStateResponse,
  FastfetchResponse,
} from "../dtos/responses/health";

import type {
  GetPhotographsResponse,
  UploadPhotographResponse,
  DeletePhotographsResponse,
} from "../dtos/responses/photography";

import type {
  GetLanguagesResponse,
  GetLanguageResponse,
  GetCountriesResponse,
  GetCountryResponse,
  GetSubdivisionsResponse,
} from "../dtos/responses/dropdown";

import type {
  IpInfo,
  GetGeoIpInfoResponse,
  GetMyIpInfoResponse,
  GeolocateResponse,
} from "../dtos/responses/geo";

import type { VisitorBoardEntry } from "../dtos/responses/visitor_board";

import type {
  CheckIfUserExistsRequest,
  LoginRequest,
  ResetPasswordRequest,
  ResetPasswordProcessRequest,
  SignupRequest,
  VerifyUserEmailRequest,
} from "../dtos/requests/auth";

import type {
  EmailValidateResponse,
  IsSuperuserResponse,
  LoginResponse,
  LogoutResponse,
  MeResponse,
  ResetPasswordRequestResponse,
  ResetPasswordResponse,
  SignupResponse,
} from "../dtos/responses/auth";

import type {
  GetPostsRequest,
  SubmitCommentRequest,
  SubmitPostRequest,
  UpvoteCommentRequest,
  UpvotePostRequest,
} from "../dtos/requests/blog";

import type {
  GetPostsResponse,
  ReadPostResponse,
  SearchPostsResponse,
  SubmitCommentResponse,
  SubmitPostResponse,
  VoteCommentResponse,
  VotePostResponse,
  DeletePostResponse,
  DeleteCommentResponse,
} from "../dtos/responses/blog";

import type { GetCountryLanguageBundleRequest } from "../dtos/requests/i18n";
import type { GetCountryLanguageBundleResponse } from "../dtos/responses/i18n";
import type { SyncI18nCacheResponse } from "../dtos/responses/admin";

/**
 * API Endpoints Grouped by Domain
 */
export const healthApi = {
  server: async () => await get<void>("/api/healthcheck/server"),
  state: async () =>
    await get<ApiResponse<HealthStateResponse>>("/api/healthcheck/state"),
  fastfetch: async () =>
    await get<ApiResponse<FastfetchResponse>>("/api/healthcheck/fastfetch"),
};

export const photographyApi = {
  getPhotographs: async (page = 1, pageSize = 20) =>
    await get<ApiResponse<GetPhotographsResponse>>(
      `/api/photographs/get?page=${page}&page_size=${pageSize}`,
    ),
  uploadPhotograph: async (
    formData: FormData,
    opts?: { onUploadProgress?: (percent: number) => void },
  ) =>
    await postFormData<ApiResponse<UploadPhotographResponse>>(
      "/api/photographs/upload",
      formData,
      {
        onUploadProgress: opts?.onUploadProgress,
      },
    ),
  deletePhotographs: async (photograph_ids: string[]) =>
    await del<ApiResponse<DeletePhotographsResponse>>(
      "/api/photographs/delete",
      { body: { photograph_ids } },
    ),
};

export const dropdownApi = {
  languageList: async () =>
    await get<ApiResponse<GetLanguagesResponse>>("/api/dropdown/language"),
  language: async (language_id: string | number) =>
    await get<ApiResponse<GetLanguageResponse>>(
      `/api/dropdown/language/{language_id}`,
      {
        params: { language_id },
      },
    ),
  countryList: (function () {
    let cache: ApiResponse<GetCountriesResponse> | null = null;
    return async () => {
      if (cache) return cache;
      cache = await get<ApiResponse<GetCountriesResponse>>(
        "/api/dropdown/country",
      );
      return cache;
    };
  })(),
  country: async (country_id: string | number) =>
    await get<ApiResponse<GetCountryResponse>>(
      "/api/dropdown/country/{country_id}",
      { params: { country_id } },
    ),
  countrySubdivisions: async (country_id: string | number) =>
    await get<ApiResponse<GetSubdivisionsResponse>>(
      "/api/dropdown/country/{country_id}/subdivision",
      {
        params: { country_id },
      },
    ),
};

export const geoApi = {
  lookupIp: async (ip_address: string) =>
    await get<ApiResponse<GeolocateResponse>>("/api/geolocate/{ip_address}", {
      params: { ip_address },
    }),
};

export const geoIpApi = {
  getGeoIpInfo: (ip: string) =>
    get<ApiResponse<GetGeoIpInfoResponse>>(`/api/geo-ip-info/${ip}`),
  getMyIpInfo: () =>
    get<ApiResponse<GetMyIpInfoResponse>>("/api/geo-ip-info/me"),
};

export const authApi = {
  signup: async (body: SignupRequest) =>
    await post<ApiResponse<SignupResponse>>("/api/auth/signup", { body }),
  checkIfUserExists: async (body: CheckIfUserExistsRequest) =>
    await post<ApiResponse<EmailValidateResponse>>(
      "/api/auth/check-if-user-exists",
      { body },
    ),
  login: async (body: LoginRequest) =>
    await post<ApiResponse<LoginResponse>>("/api/auth/login", { body }),
  resetPasswordRequest: async (body: ResetPasswordRequest) =>
    await post<ApiResponse<ResetPasswordRequestResponse>>(
      "/api/auth/reset-password-request",
      { body },
    ),
  resetPassword: async (body: ResetPasswordProcessRequest) =>
    await post<ApiResponse<ResetPasswordResponse>>("/api/auth/reset-password", {
      body,
    }),
  verifyUserEmail: async (body: VerifyUserEmailRequest) =>
    await post<ApiResponse<EmailValidateResponse>>(
      "/api/auth/verify-user-email",
      { body },
    ),
  // me: gets user info, but also reads server build info from headers for overlay
  me: async (): Promise<ApiResponse<MeResponse>> => {
    const url = interpolate("/api/auth/me", {});
    const res = await apiFetch(url, { method: "GET" });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  isSuperuser: async () =>
    await get<ApiResponse<IsSuperuserResponse>>("/api/auth/is-superuser"),
  logout: async () =>
    await post<ApiResponse<LogoutResponse>>("/api/auth/logout"),
  uploadProfilePicture: async (body: FormData) =>
    await apiFetch("/api/user/profile-picture", {
      method: "POST",
      body,
      credentials: "include",
    }).then(async (res) => {
      if (!res.ok) throw new Error(await res.text());
      return res.json();
    }),
};

export const userApi = {
  uploadProfilePicture: async (body: FormData) =>
    await apiFetch("/api/user/upload-profile-picture", {
      method: "POST",
      body,
      credentials: "include",
    }).then(async (res) => {
      if (!res.ok) throw new Error(await res.text());
      return res.json();
    }),
};

export const blogApi = {
  getPosts: async (query?: GetPostsRequest) =>
    await get<ApiResponse<GetPostsResponse>>("/api/blog/posts", {
      params: query ? { ...query } : undefined,
    }),
  readPost: async (post_id: string) =>
    await get<ApiResponse<ReadPostResponse>>("/api/blog/posts/{post_id}", {
      params: { post_id },
    }),
  submitPost: async (body: SubmitPostRequest) =>
    await post<ApiResponse<SubmitPostResponse>>("/api/blog/posts", { body }),
  updatePost: async (
    body: Omit<SubmitPostRequest, "post_id">,
    post_id: string,
  ) =>
    await patch<ApiResponse<SubmitPostResponse>>("/api/blog/{post_id}", {
      body,
      params: { post_id },
    }),
  // Add voting and commenting endpoints
  votePost: async (body: UpvotePostRequest, post_id: string) =>
    await post<ApiResponse<VotePostResponse>>("/api/blog/{post_id}/vote", {
      body,
      params: { post_id },
    }),
  voteComment: async (
    body: UpvoteCommentRequest,
    post_id: string,
    comment_id: string,
  ) =>
    await post<ApiResponse<VoteCommentResponse>>(
      "/api/blog/{post_id}/{comment_id}/vote",
      { body, params: { post_id, comment_id } },
    ),
  rescindPostVote: async (post_id: string) =>
    await del<ApiResponse<VotePostResponse>>("/api/blog/{post_id}/vote", {
      params: { post_id },
    }),
  rescindCommentVote: async (post_id: string, comment_id: string) =>
    await del<ApiResponse<VoteCommentResponse>>(
      "/api/blog/{post_id}/{comment_id}/vote",
      { params: { post_id, comment_id } },
    ),
  submitComment: async (body: SubmitCommentRequest, post_id: string) =>
    await post<ApiResponse<SubmitCommentResponse>>(
      "/api/blog/{post_id}/comment",
      { body, params: { post_id } },
    ),
  updateComment: async (
    body: { comment_content: string },
    post_id: string,
    comment_id: string,
  ) =>
    await patch<ApiResponse<SubmitCommentResponse>>(
      "/api/blog/{post_id}/{comment_id}",
      { body, params: { post_id, comment_id } },
    ),
  deletePost: async (post_id: string) =>
    await del<ApiResponse<DeletePostResponse>>("/api/blog/{post_id}", {
      params: { post_id },
    }),
  deleteComment: async (post_id: string, comment_id: string) =>
    await del<ApiResponse<DeleteCommentResponse>>(
      "/api/blog/{post_id}/{comment_id}",
      { params: { post_id, comment_id } },
    ),
  searchPosts: async (
    query: string,
    searchType: "title" | "tag" = "title",
    page = 1,
    limit = 20,
    tags?: string[],
  ) =>
    await get<ApiResponse<SearchPostsResponse>>(
      `/api/blog/search?q=${encodeURIComponent(query)}&search_type=${searchType}&page=${page}&limit=${limit}${
        tags && tags.length > 0
          ? `&tags=${encodeURIComponent(tags.join(","))}`
          : ""
      }`,
    ),
};

export const i18nApi = {
  getCountryLanguageBundle: async (query?: GetCountryLanguageBundleRequest) =>
    await get<ApiResponse<GetCountryLanguageBundleResponse>>(
      "/api/i18n/country-language-bundle",
      { params: query ? { ...query } : undefined },
    ),
};

export const adminApi = {
  syncCountryLanguageBundle: async () =>
    await get<ApiResponse<SyncI18nCacheResponse>>(
      "/api/admin/sync-country-language-bundle",
    ),
};

export const visitorBoardApi = {
  getVisitorBoard: async () =>
    await get<ApiResponse<VisitorBoardEntry[]>>("/api/visitor-board"),
};

// WASM Module types
export interface WasmModuleItem {
  wasm_module_id: string;
  user_id: string;
  wasm_module_title: string;
  wasm_module_description: string;
  wasm_module_link: string;
  wasm_module_thumbnail_link: string;
  wasm_module_created_at: string;
  wasm_module_updated_at: string;
}

export interface GetWasmModulesResponse {
  items: WasmModuleItem[];
}

export interface UpdateWasmModuleRequest {
  wasm_module_title?: string;
  wasm_module_description?: string;
}

export const wasmModuleApi = {
  getWasmModules: async () =>
    await get<ApiResponse<GetWasmModulesResponse>>("/api/wasm-modules"),

  uploadWasmModule: async (
    formData: FormData,
    opts?: { onUploadProgress?: (percent: number) => void },
  ) =>
    await postFormData<ApiResponse<WasmModuleItem>>(
      "/api/wasm-modules",
      formData,
      { onUploadProgress: opts?.onUploadProgress },
    ),

  updateWasmModule: async (
    wasm_module_id: string,
    body: UpdateWasmModuleRequest,
  ) =>
    await patch<ApiResponse<WasmModuleItem>>(
      "/api/wasm-modules/{wasm_module_id}",
      { body, params: { wasm_module_id } },
    ),

  updateWasmModuleAssets: async (
    wasm_module_id: string,
    formData: FormData,
    opts?: { onUploadProgress?: (percent: number) => void },
  ) =>
    await postFormData<ApiResponse<WasmModuleItem>>(
      "/api/wasm-modules/{wasm_module_id}/assets",
      formData,
      { params: { wasm_module_id }, onUploadProgress: opts?.onUploadProgress },
    ),

  deleteWasmModule: async (wasm_module_id: string) =>
    await del<ApiResponse<{ deleted_wasm_module_id: string }>>(
      "/api/wasm-modules/{wasm_module_id}",
      { params: { wasm_module_id } },
    ),
};

export { get, post, patch, del, interpolate };

// Re-export commonly used types for convenience
export type { IpInfo };

/**
 * Usage Example:
 * import { authApi } from "./all_api";
 * await authApi.login({ user_email, user_password });
 *
 * // For path with params:
 * await dropdownApi.language(5);
 * await blogApi.readPost("abc-123");
 */
