// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  CreateForumReplyRequest,
  CreateForumTopicRequest,
  DeleteForumContentRequest,
  ForumCapabilitiesResponse,
  ForumModerationAuditListResponse,
  ForumModerationResponse,
  ForumNotificationListResponse,
  ForumNotificationReadResponse,
  ForumReplyMutationResponse,
  ForumSubscriptionResponse,
  ForumTopicDetailResponse,
  ForumTopicListResponse,
  ForumTopicMutationResponse,
  ModerateForumReplyRequest,
  ModerateForumTopicRequest,
  UpdateForumReplyRequest,
  UpdateForumTopicRequest,
} from "../api-types";
import {
  appendQuery,
  interpolatePath,
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createForumClient(transport: ApiTransport) {
  return {
    createForumReply: async (input: {
      readonly body: CreateForumReplyRequest;
      readonly path: {
        readonly topic_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/forum/topics/{topic_id}/replies", input.path);
      const url = path;
      return requestJson<ApiResponse<ForumReplyMutationResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    createForumTopic: async (input: {
      readonly body: CreateForumTopicRequest;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/forum/topics";
      const url = path;
      return requestJson<ApiResponse<ForumTopicMutationResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    deleteForumReply: async (input: {
      readonly body: DeleteForumContentRequest;
      readonly path: {
        readonly reply_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/forum/replies/{reply_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<ForumReplyMutationResponse>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    deleteForumTopic: async (input: {
      readonly body: DeleteForumContentRequest;
      readonly path: {
        readonly topic_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/forum/topics/{topic_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<ForumTopicMutationResponse>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    forumCapabilities: async (options: ApiRequestOptions = {}) => {
      const path = "/api/forum/capabilities";
      const url = path;
      return requestJson<ApiResponse<ForumCapabilitiesResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    getForumTopic: async (input: {
      readonly path: {
        readonly topic_id: string;
      };
      readonly query?: {
        readonly after_reply_created_at?: string;
        readonly after_reply_id?: string;
        readonly reply_limit?: number;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/forum/topics/{topic_id}", input.path);
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<ForumTopicDetailResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    listForumModerationAudit: async (input: {
      readonly query?: {
        readonly before_audit_id?: string;
        readonly before_created_at?: string;
        readonly limit?: number;
      };
    } = {}, options: ApiRequestOptions = {}) => {
      const path = "/api/forum/moderation/audit";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<ForumModerationAuditListResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    listForumNotifications: async (input: {
      readonly query?: {
        readonly before_created_at?: string;
        readonly before_notification_id?: string;
        readonly limit?: number;
      };
    } = {}, options: ApiRequestOptions = {}) => {
      const path = "/api/forum/notifications";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<ForumNotificationListResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    listForumTopics: async (input: {
      readonly query?: {
        readonly before_activity_at?: string;
        readonly before_pinned?: boolean;
        readonly before_topic_id?: string;
        readonly limit?: number;
        readonly search?: string;
      };
    } = {}, options: ApiRequestOptions = {}) => {
      const path = "/api/forum/topics";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<ForumTopicListResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    markForumNotificationRead: async (input: {
      readonly path: {
        readonly notification_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/forum/notifications/{notification_id}/read", input.path);
      const url = path;
      return requestJson<ApiResponse<ForumNotificationReadResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    moderateForumReply: async (input: {
      readonly body: ModerateForumReplyRequest;
      readonly path: {
        readonly reply_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/forum/replies/{reply_id}/moderation", input.path);
      const url = path;
      return requestJson<ApiResponse<ForumModerationResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    moderateForumTopic: async (input: {
      readonly body: ModerateForumTopicRequest;
      readonly path: {
        readonly topic_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/forum/topics/{topic_id}/moderation", input.path);
      const url = path;
      return requestJson<ApiResponse<ForumModerationResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    subscribeForumTopic: async (input: {
      readonly path: {
        readonly topic_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/forum/topics/{topic_id}/subscription", input.path);
      const url = path;
      return requestJson<ApiResponse<ForumSubscriptionResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    unsubscribeForumTopic: async (input: {
      readonly path: {
        readonly topic_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/forum/topics/{topic_id}/subscription", input.path);
      const url = path;
      return requestJson<ApiResponse<ForumSubscriptionResponse>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    updateForumReply: async (input: {
      readonly body: UpdateForumReplyRequest;
      readonly path: {
        readonly reply_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/forum/replies/{reply_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<ForumReplyMutationResponse>>(transport, url, {
        method: "PATCH",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    updateForumTopic: async (input: {
      readonly body: UpdateForumTopicRequest;
      readonly path: {
        readonly topic_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/forum/topics/{topic_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<ForumTopicMutationResponse>>(transport, url, {
        method: "PATCH",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
  } as const;
}
