export interface GetLiveChatMessagesRequest {
  limit?: number;
  before_message_id?: string;
}

export type LiveChatClientEvent =
  | {
      type: "send_message";
      client_message_id: string;
      body: string;
    }
  | {
      type: "typing";
      is_typing: boolean;
    }
  | {
      type: "heartbeat";
      nonce: string;
    };
