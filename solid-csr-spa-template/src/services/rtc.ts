import type { RtcClientSignal } from "../dtos/requests/live_chat";
import {
  encodeRtcAnswerFrame,
  encodeRtcIceFrame,
  encodeRtcJoinFrame,
  encodeRtcLeaveFrame,
  encodeRtcMediaStateFrame,
} from "./live_chat_binary";

// Codec preference order. The SFU forwards opaque RTP (no transcoding), so the
// negotiated codec must be one both ends support; AV1 is preferred where
// available, then VP9/VP8, with H264 last. Audio is always Opus.
const VIDEO_CODEC_PREFERENCE = ["video/av1", "video/vp9", "video/vp8", "video/h264"];
const AUDIO_CODEC_PREFERENCE = ["audio/opus"];

/// Create a peer connection. No STUN/TURN by default: the SFU advertises its
/// public IP as a host candidate, so clients dial it directly.
export function createPeerConnection(): RTCPeerConnection {
  return new RTCPeerConnection({ iceServers: [] });
}

/// Request microphone + camera. Rejects if permission is denied.
export async function requestUserMedia(): Promise<MediaStream> {
  return await navigator.mediaDevices.getUserMedia({
    audio: {
      echoCancellation: true,
      noiseSuppression: true,
      autoGainControl: true,
    },
    video: {
      width: { ideal: 1280 },
      height: { ideal: 720 },
      frameRate: { ideal: 30 },
    },
  });
}

/// Apply codec preferences per transceiver. Best-effort: browsers without
/// `setCodecPreferences` (or a given codec) silently keep their defaults.
export function applyCodecPreferences(pc: RTCPeerConnection): void {
  try {
    for (const transceiver of pc.getTransceivers()) {
      const kind = transceiver.receiver.track?.kind ?? transceiver.sender.track?.kind;
      if (kind !== "audio" && kind !== "video") continue;
      if (typeof transceiver.setCodecPreferences !== "function") continue;
      const capabilities = RTCRtpReceiver.getCapabilities(kind);
      if (!capabilities) continue;
      const order = kind === "video" ? VIDEO_CODEC_PREFERENCE : AUDIO_CODEC_PREFERENCE;
      const sorted = [...capabilities.codecs].sort(
        (a, b) => codecRank(a.mimeType, order) - codecRank(b.mimeType, order),
      );
      transceiver.setCodecPreferences(sorted);
    }
  } catch (err) {
    console.warn("Failed to apply codec preferences:", err);
  }
}

function codecRank(mimeType: string, order: string[]): number {
  const index = order.indexOf(mimeType.toLowerCase());
  return index === -1 ? order.length : index;
}

/// Convert a browser ICE candidate into a client signal for the wire.
export function iceCandidateToSignal(candidate: RTCIceCandidate): RtcClientSignal {
  return {
    kind: "ice",
    candidate: candidate.candidate,
    sdp_mid: candidate.sdpMid,
    sdp_mline_index: candidate.sdpMLineIndex,
  };
}

/// Convert a received ICE signal into an `RTCIceCandidateInit` for `addIceCandidate`.
export function signalToIceInit(signal: {
  candidate: string;
  sdp_mid: string | null;
  sdp_mline_index: number | null;
}): RTCIceCandidateInit {
  return {
    candidate: signal.candidate,
    sdpMid: signal.sdp_mid ?? undefined,
    sdpMLineIndex: signal.sdp_mline_index ?? undefined,
  };
}

/// Encode a client RTC signal to a binary frame for the negotiated protocol.
export function encodeRtcClientFrame(signal: RtcClientSignal): ArrayBuffer {
  switch (signal.kind) {
    case "join":
      return encodeRtcJoinFrame(signal.sdp, signal.want_audio, signal.want_video);
    case "answer":
      return encodeRtcAnswerFrame(signal.sdp);
    case "ice":
      return encodeRtcIceFrame(
        signal.candidate,
        signal.sdp_mid,
        signal.sdp_mline_index,
      );
    case "leave":
      return encodeRtcLeaveFrame();
    case "media_state":
      return encodeRtcMediaStateFrame(signal.mic_on, signal.cam_on);
  }
}

/// Stable stream id the SFU stamps on a publisher's tracks, matching the Rust
/// `actor_stream_id`. Used to map a remote `MediaStream` back to its actor.
export function actorStreamId(actorKey: {
  type: "user" | "guest";
  value: string;
}): string {
  return `${actorKey.type}:${actorKey.value}`;
}
