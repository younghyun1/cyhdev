// Generated from rust-be-template OpenAPI. Do not edit by hand.


export type LiveChatMessageItem = {
  readonly guest_ip?: string | null;
  readonly live_chat_message_id: string;
  readonly message_body: string;
  readonly message_created_at: string;
  readonly message_deleted_at?: string | null;
  readonly message_edited_at?: string | null;
  readonly room_key: string;
  readonly sender_country_flag?: string | null;
  readonly sender_display_name: string;
  readonly sender_kind: number;
  readonly user_id?: string | null;
  readonly user_profile_picture_url?: string | null;
};
