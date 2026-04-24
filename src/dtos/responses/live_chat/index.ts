export type ChatActorKey =
  | {
      type: "user";
      value: string;
    }
  | {
      type: "guest";
      value: string;
    };

export interface ChatActor {
  actor_key: ChatActorKey;
  sender_kind: number;
  user_id: string | null;
  guest_ip: string | null;
  display_name: string;
  country_flag: string | null;
  user_profile_picture_url: string | null;
}

export interface LiveChatMessageItem {
  live_chat_message_id: string;
  room_key: string;
  user_id: string | null;
  guest_ip: string | null;
  sender_kind: number;
  sender_display_name: string;
  sender_country_flag: string | null;
  user_profile_picture_url: string | null;
  message_body: string;
  message_created_at: string;
  message_edited_at: string | null;
  message_deleted_at: string | null;
}

export interface GetLiveChatMessagesResponse {
  items: LiveChatMessageItem[];
  next_before_message_id: string | null;
  has_more: boolean;
}

export interface LiveChatCacheStatsResponse {
  max_bytes: number;
  used_bytes: number;
  message_count: number;
  oldest_cached_at: string | null;
  newest_cached_at: string | null;
  active_typing_count: number;
  connected_count: number;
}

export type LiveChatServerEvent =
  | {
      type: "hello";
      actor: ChatActor;
      recent_messages: LiveChatMessageItem[];
      connected_count: number;
    }
  | {
      type: "message";
      message: LiveChatMessageItem;
    }
  | {
      type: "message_ack";
      client_message_id: string;
      message: LiveChatMessageItem;
    }
  | {
      type: "typing";
      actor: ChatActor;
      is_typing: boolean;
      expires_at: string;
    }
  | {
      type: "presence";
      connected_count: number;
    }
  | {
      type: "heartbeat_ack";
      nonce: string;
    }
  | {
      type: "error";
      code: string;
      message: string;
    };
