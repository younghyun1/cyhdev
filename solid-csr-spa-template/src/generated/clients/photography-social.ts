// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  DeletePhotographCommentResponse,
  PhotographCommentResponse,
  SubmitPhotographCommentRequest,
  UpdatePhotographCommentRequest,
  VotePhotographRequest,
  VotePhotographResponse,
} from "../api-types";
import {
  interpolatePath,
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createPhotographySocialClient(transport: ApiTransport) {
  return {
    deletePhotographComment: async (input: {
      readonly path: {
        readonly comment_id: string;
        readonly photograph_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/photographs/{photograph_id}/{comment_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<DeletePhotographCommentResponse>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    rescindPhotographCommentVote: async (input: {
      readonly path: {
        readonly comment_id: string;
        readonly photograph_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/photographs/{photograph_id}/{comment_id}/vote", input.path);
      const url = path;
      return requestJson<ApiResponse<null>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    rescindPhotographVote: async (input: {
      readonly path: {
        readonly photograph_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/photographs/{photograph_id}/vote", input.path);
      const url = path;
      return requestJson<ApiResponse<null>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    submitPhotographComment: async (input: {
      readonly body: SubmitPhotographCommentRequest;
      readonly path: {
        readonly photograph_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/photographs/{photograph_id}/comment", input.path);
      const url = path;
      return requestJson<ApiResponse<PhotographCommentResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    updatePhotographComment: async (input: {
      readonly body: UpdatePhotographCommentRequest;
      readonly path: {
        readonly comment_id: string;
        readonly photograph_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/photographs/{photograph_id}/{comment_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<PhotographCommentResponse>>(transport, url, {
        method: "PATCH",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    votePhotograph: async (input: {
      readonly body: VotePhotographRequest;
      readonly path: {
        readonly photograph_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/photographs/{photograph_id}/vote", input.path);
      const url = path;
      return requestJson<ApiResponse<VotePhotographResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    votePhotographComment: async (input: {
      readonly body: VotePhotographRequest;
      readonly path: {
        readonly comment_id: string;
        readonly photograph_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/photographs/{photograph_id}/{comment_id}/vote", input.path);
      const url = path;
      return requestJson<ApiResponse<VotePhotographResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
  } as const;
}
