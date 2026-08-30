// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  DeletePostResponse,
  GetPostsResponse,
  ReadPostResponse,
  SearchPostsResponse,
  SubmitPostRequest,
  SubmitPostResponse,
  UpdatePostRequest,
} from "../api-types";
import {
  appendQuery,
  interpolatePath,
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createBlogPostsClient(transport: ApiTransport) {
  return {
    deletePost: async (input: {
      readonly path: {
        readonly post_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/blog/{post_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<DeletePostResponse>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    getPosts: async (input: {
      readonly query?: {
        readonly page?: number;
        readonly posts_per_page?: number;
      };
    } = {}, options: ApiRequestOptions = {}) => {
      const path = "/api/blog/posts";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<GetPostsResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    readPost: async (input: {
      readonly path: {
        readonly post_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/blog/posts/{post_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<ReadPostResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    searchPosts: async (input: {
      readonly query: {
        readonly limit?: number;
        readonly page?: number;
        readonly q: string;
        readonly search_type?: string;
        readonly tags?: string | null;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/blog/search";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<SearchPostsResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    submitPost: async (input: {
      readonly body: SubmitPostRequest;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/blog/posts";
      const url = path;
      return requestJson<ApiResponse<SubmitPostResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    updatePost: async (input: {
      readonly body: UpdatePostRequest;
      readonly path: {
        readonly post_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/blog/{post_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<SubmitPostResponse>>(transport, url, {
        method: "PATCH",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
  } as const;
}
