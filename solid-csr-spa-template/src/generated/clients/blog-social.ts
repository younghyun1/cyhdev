// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  CommentResponse,
  DeleteCommentResponse,
  SubmitCommentRequest,
  UpdateCommentRequest,
  UpvoteCommentRequest,
  UpvotePostRequest,
  VoteCommentResponse,
  VotePostResponse,
} from "../api-types";
import {
  interpolatePath,
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createBlogSocialClient(transport: ApiTransport) {
  return {
    deleteComment: async (input: {
      readonly path: {
        readonly comment_id: string;
        readonly post_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/blog/{post_id}/{comment_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<DeleteCommentResponse>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    rescindCommentVote: async (input: {
      readonly path: {
        readonly comment_id: string;
        readonly post_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/blog/{post_id}/{comment_id}/vote", input.path);
      const url = path;
      return requestJson<ApiResponse<null>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    rescindPostVote: async (input: {
      readonly path: {
        readonly post_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/blog/{post_id}/vote", input.path);
      const url = path;
      return requestJson<ApiResponse<null>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    submitComment: async (input: {
      readonly body: SubmitCommentRequest;
      readonly path: {
        readonly post_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/blog/{post_id}/comment", input.path);
      const url = path;
      return requestJson<ApiResponse<CommentResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    updateComment: async (input: {
      readonly body: UpdateCommentRequest;
      readonly path: {
        readonly comment_id: string;
        readonly post_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/blog/{post_id}/{comment_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<CommentResponse>>(transport, url, {
        method: "PATCH",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    voteComment: async (input: {
      readonly body: UpvoteCommentRequest;
      readonly path: {
        readonly comment_id: string;
        readonly post_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/blog/{post_id}/{comment_id}/vote", input.path);
      const url = path;
      return requestJson<ApiResponse<VoteCommentResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    votePost: async (input: {
      readonly body: UpvotePostRequest;
      readonly path: {
        readonly post_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/blog/{post_id}/vote", input.path);
      const url = path;
      return requestJson<ApiResponse<VotePostResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
  } as const;
}
