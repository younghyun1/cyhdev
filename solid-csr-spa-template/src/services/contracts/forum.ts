import { ApiContractError, type ApiResponse } from "../../generated";
import { apiFetch } from "../api";
import type {
  CreateForumReplyRequest,
  CreateForumTopicRequest,
  DeleteForumContentRequest,
  ForumCapabilitiesResponse,
  ForumModerationAuditCursor,
  ForumModerationAuditListResponse,
  ForumModerationResponse,
  ForumNotificationCursor,
  ForumNotificationListResponse,
  ForumNotificationReadResponse,
  ForumReplyCursor,
  ForumReplyMutationResponse,
  ForumSubscriptionResponse,
  ForumTopicCursor,
  ForumTopicDetailResponse,
  ForumTopicListResponse,
  ForumTopicMutationResponse,
  ModerateForumReplyRequest,
  ModerateForumTopicRequest,
  UpdateForumReplyRequest,
  UpdateForumTopicRequest,
} from "./forum_types";

async function forumRequest<T>(
  path: string,
  init: RequestInit = {},
): Promise<ApiResponse<T>> {
  const response = await apiFetch(path, init);
  const raw = await response.text();
  if (!response.ok) throw new ApiContractError(response.status, raw);
  try {
    return JSON.parse(raw) as ApiResponse<T>;
  } catch {
    throw new Error("Forum API returned malformed JSON");
  }
}

function jsonRequest(method: string, body: unknown): RequestInit {
  return {
    method,
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  };
}

function addCursor(
  query: URLSearchParams,
  cursor:
    | ForumTopicCursor
    | ForumReplyCursor
    | ForumNotificationCursor
    | ForumModerationAuditCursor
    | null,
): void {
  if (!cursor) return;
  for (const [key, value] of Object.entries(cursor)) {
    query.set(key, String(value));
  }
}

function forumPath(path: string, query?: URLSearchParams): string {
  const encoded = query?.toString();
  return encoded ? `${path}?${encoded}` : path;
}

const encoded = (value: string) => encodeURIComponent(value);

export const forumApi = {
  capabilities: () =>
    forumRequest<ForumCapabilitiesResponse>("/api/forum/capabilities"),
  topics: (
    search: string,
    cursor: ForumTopicCursor | null,
    limit = 25,
  ) => {
    const query = new URLSearchParams({ limit: String(limit) });
    if (search.trim()) query.set("search", search.trim());
    addCursor(query, cursor);
    return forumRequest<ForumTopicListResponse>(
      forumPath("/api/forum/topics", query),
    );
  },
  topic: (topicId: string, cursor: ForumReplyCursor | null, limit = 50) => {
    const query = new URLSearchParams({ reply_limit: String(limit) });
    addCursor(query, cursor);
    return forumRequest<ForumTopicDetailResponse>(
      forumPath(`/api/forum/topics/${encoded(topicId)}`, query),
    );
  },
  createTopic: (body: CreateForumTopicRequest) =>
    forumRequest<ForumTopicMutationResponse>(
      "/api/forum/topics",
      jsonRequest("POST", body),
    ),
  updateTopic: (topicId: string, body: UpdateForumTopicRequest) =>
    forumRequest<ForumTopicMutationResponse>(
      `/api/forum/topics/${encoded(topicId)}`,
      jsonRequest("PATCH", body),
    ),
  deleteTopic: (topicId: string, body: DeleteForumContentRequest) =>
    forumRequest<ForumTopicMutationResponse>(
      `/api/forum/topics/${encoded(topicId)}`,
      jsonRequest("DELETE", body),
    ),
  createReply: (topicId: string, body: CreateForumReplyRequest) =>
    forumRequest<ForumReplyMutationResponse>(
      `/api/forum/topics/${encoded(topicId)}/replies`,
      jsonRequest("POST", body),
    ),
  updateReply: (replyId: string, body: UpdateForumReplyRequest) =>
    forumRequest<ForumReplyMutationResponse>(
      `/api/forum/replies/${encoded(replyId)}`,
      jsonRequest("PATCH", body),
    ),
  deleteReply: (replyId: string, body: DeleteForumContentRequest) =>
    forumRequest<ForumReplyMutationResponse>(
      `/api/forum/replies/${encoded(replyId)}`,
      jsonRequest("DELETE", body),
    ),
  subscribe: (topicId: string) =>
    forumRequest<ForumSubscriptionResponse>(
      `/api/forum/topics/${encoded(topicId)}/subscription`,
      { method: "POST" },
    ),
  unsubscribe: (topicId: string) =>
    forumRequest<ForumSubscriptionResponse>(
      `/api/forum/topics/${encoded(topicId)}/subscription`,
      { method: "DELETE" },
    ),
  notifications: (cursor: ForumNotificationCursor | null, limit = 50) => {
    const query = new URLSearchParams({ limit: String(limit) });
    addCursor(query, cursor);
    return forumRequest<ForumNotificationListResponse>(
      forumPath("/api/forum/notifications", query),
    );
  },
  readNotification: (notificationId: string) =>
    forumRequest<ForumNotificationReadResponse>(
      `/api/forum/notifications/${encoded(notificationId)}/read`,
      { method: "POST" },
    ),
  moderateTopic: (topicId: string, body: ModerateForumTopicRequest) =>
    forumRequest<ForumModerationResponse>(
      `/api/forum/topics/${encoded(topicId)}/moderation`,
      jsonRequest("POST", body),
    ),
  moderateReply: (replyId: string, body: ModerateForumReplyRequest) =>
    forumRequest<ForumModerationResponse>(
      `/api/forum/replies/${encoded(replyId)}/moderation`,
      jsonRequest("POST", body),
    ),
  moderationAudit: (
    cursor: ForumModerationAuditCursor | null,
    limit = 50,
  ) => {
    const query = new URLSearchParams({ limit: String(limit) });
    addCursor(query, cursor);
    return forumRequest<ForumModerationAuditListResponse>(
      forumPath("/api/forum/moderation/audit", query),
    );
  },
} as const;

export type * from "./forum_types";
