/** TypeScript mirror of the backend forum HTTP DTO boundary. */

export type ForumContentState = "visible" | "hidden" | "deleted";
export type ForumTopicAccessState = "open" | "locked";
export type ForumNotificationKind = "topic_reply";
export type ForumTopicModerationAction =
  | "hide"
  | "restore"
  | "lock"
  | "unlock"
  | "pin"
  | "unpin";
export type ForumReplyModerationAction = "hide" | "restore";
export type ForumModerationAction =
  | "topic_hidden"
  | "topic_restored"
  | "topic_locked"
  | "topic_unlocked"
  | "topic_pinned"
  | "topic_unpinned"
  | "reply_hidden"
  | "reply_restored";

export interface ForumAuthor {
  readonly public_user_id: string;
  readonly display_name: string;
  readonly country_code: number | null;
  readonly profile_picture_url: string | null;
  readonly is_deleted: boolean;
}

export interface ForumTopic {
  readonly topic_id: string;
  readonly author: ForumAuthor;
  readonly title: string | null;
  readonly body: string | null;
  readonly content_state: ForumContentState;
  readonly access_state: ForumTopicAccessState;
  readonly is_pinned: boolean;
  readonly revision: number;
  readonly reply_count: number;
  readonly created_at: string;
  readonly updated_at: string;
  readonly last_activity_at: string;
  readonly edited_at: string | null;
}

export interface ForumReply {
  readonly reply_id: string;
  readonly topic_id: string;
  readonly author: ForumAuthor;
  readonly body: string | null;
  readonly content_state: ForumContentState;
  readonly revision: number;
  readonly created_at: string;
  readonly updated_at: string;
  readonly edited_at: string | null;
}

export interface ForumTopicCursor {
  readonly before_pinned: boolean;
  readonly before_activity_at: string;
  readonly before_topic_id: string;
}

export interface ForumReplyCursor {
  readonly after_reply_created_at: string;
  readonly after_reply_id: string;
}

export interface ForumNotificationCursor {
  readonly before_created_at: string;
  readonly before_notification_id: string;
}

export interface ForumModerationAuditCursor {
  readonly before_created_at: string;
  readonly before_audit_id: string;
}

export interface ForumTopicListResponse {
  readonly topics: ReadonlyArray<ForumTopic>;
  readonly next_cursor: ForumTopicCursor | null;
}

export interface ForumTopicDetailResponse {
  readonly topic: ForumTopic;
  readonly replies: ReadonlyArray<ForumReply>;
  readonly next_reply_cursor: ForumReplyCursor | null;
  readonly is_subscribed: boolean;
}

export interface ForumCapabilitiesResponse {
  readonly authenticated: boolean;
  readonly can_post: boolean;
  readonly can_moderate: boolean;
}

export interface ForumTopicMutationResponse {
  readonly topic_id: string;
  readonly revision: number;
  readonly updated_at: string;
}

export interface ForumReplyMutationResponse {
  readonly reply_id: string;
  readonly revision: number;
  readonly updated_at: string;
}

export interface ForumSubscriptionResponse {
  readonly topic_id: string;
  readonly subscribed: boolean;
}

export interface ForumNotification {
  readonly notification_id: string;
  readonly actor: ForumAuthor;
  readonly topic_id: string;
  readonly reply_id: string;
  readonly kind: ForumNotificationKind;
  readonly topic_title: string | null;
  readonly created_at: string;
  readonly expires_at: string;
  readonly read_at: string | null;
}

export interface ForumNotificationListResponse {
  readonly notifications: ReadonlyArray<ForumNotification>;
  readonly next_cursor: ForumNotificationCursor | null;
}

export interface ForumNotificationReadResponse {
  readonly notification_id: string;
  readonly read_at: string;
}

export interface ForumModerationResponse {
  readonly audit_event_id: string;
  readonly target_id: string;
  readonly revision: number;
  readonly action: ForumModerationAction;
  readonly moderated_at: string;
}

export interface ForumModerationAuditItem {
  readonly audit_event_id: string;
  readonly actor: ForumAuthor;
  readonly topic_id: string | null;
  readonly reply_id: string | null;
  readonly action: ForumModerationAction;
  readonly reason: string;
  readonly request_id: string | null;
  readonly created_at: string;
}

export interface ForumModerationAuditListResponse {
  readonly events: ReadonlyArray<ForumModerationAuditItem>;
  readonly next_cursor: ForumModerationAuditCursor | null;
}

export interface CreateForumTopicRequest {
  readonly title: string;
  readonly body: string;
}

export interface UpdateForumTopicRequest extends CreateForumTopicRequest {
  readonly expected_revision: number;
}

export interface DeleteForumContentRequest {
  readonly expected_revision: number;
}

export interface CreateForumReplyRequest {
  readonly body: string;
}

export interface UpdateForumReplyRequest extends CreateForumReplyRequest {
  readonly expected_revision: number;
}

export interface ModerateForumTopicRequest {
  readonly action: ForumTopicModerationAction;
  readonly reason: string;
  readonly expected_revision: number;
}

export interface ModerateForumReplyRequest {
  readonly action: ForumReplyModerationAction;
  readonly reason: string;
  readonly expected_revision: number;
}
