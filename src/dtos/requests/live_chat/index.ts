export interface GetLiveChatMessagesRequest {
  limit?: number;
  before_message_id?: string;
}

/// WebRTC signaling sent from the client. Mirrors the Rust `RtcClientSignal`
/// (serde internally tagged with "kind"). Over JSON these are flattened under
/// the live-chat client event as `{ type: "rtc", kind, ... }`.
export type RtcClientSignal =
  | { kind: "join"; sdp: string; want_audio: boolean; want_video: boolean }
  | { kind: "answer"; sdp: string }
  | {
      kind: "ice";
      candidate: string;
      sdp_mid: string | null;
      sdp_mline_index: number | null;
    }
  | { kind: "leave" }
  | { kind: "media_state"; mic_on: boolean; cam_on: boolean };

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
    }
  | ({ type: "rtc" } & RtcClientSignal);
