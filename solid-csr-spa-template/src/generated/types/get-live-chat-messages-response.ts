// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { LiveChatMessageItem } from "./live-chat-message-item";

export type GetLiveChatMessagesResponse = {
  readonly has_more: boolean;
  readonly items: ReadonlyArray<LiveChatMessageItem>;
  readonly next_before_message_id?: string | null;
};
