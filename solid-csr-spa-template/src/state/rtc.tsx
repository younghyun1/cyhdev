import {
  type Accessor,
  type ParentComponent,
  createContext,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  useContext,
} from "solid-js";
import type { RtcClientSignal } from "../dtos/requests/live_chat";
import type {
  ChatActor,
  RtcParticipant,
  RtcServerSignal,
} from "../dtos/responses/live_chat";
import {
  actorStreamId,
  applyCodecPreferences,
  createPeerConnection,
  encodeRtcClientFrame,
  iceCandidateToSignal,
  requestUserMedia,
  signalToIceInit,
} from "../services/rtc";
import { useLiveChatSocket } from "./live_chat_socket";

export type CallState = "idle" | "joining" | "in_call" | "error";

export interface RemoteTile {
  streamId: string;
  participant: RtcParticipant;
  stream: MediaStream | null;
}

interface RtcContextValue {
  callState: Accessor<CallState>;
  callError: Accessor<string | null>;
  localStream: Accessor<MediaStream | null>;
  micOn: Accessor<boolean>;
  camOn: Accessor<boolean>;
  selfActor: Accessor<ChatActor | null>;
  remoteTiles: Accessor<RemoteTile[]>;
  participantCount: Accessor<number>;
  joinCall: () => Promise<void>;
  leaveCall: () => void;
  toggleMic: () => void;
  toggleCamera: () => void;
}

const RtcContext = createContext<RtcContextValue>();

// Error codes that make the local call unrecoverable; the client resets on them.
const FATAL_RTC_ERROR_CODES = new Set([
  "rtc_disabled",
  "rtc_unavailable",
  "room_full",
  "sdp_failed",
  "persist_failed",
  "banned",
]);

function streamIdOf(actor: ChatActor): string {
  return actorStreamId(actor.actor_key);
}

/**
 * Owns the WebRTC peer connection and call state. Signaling rides the shared
 * live-chat socket. Perfect-negotiation-lite: the client offers once on join
 * and only answers SFU offers thereafter.
 */
export const RtcProvider: ParentComponent = (props) => {
  const socket = useLiveChatSocket();

  const [callState, setCallState] = createSignal<CallState>("idle");
  const [callError, setCallError] = createSignal<string | null>(null);
  const [localStream, setLocalStream] = createSignal<MediaStream | null>(null);
  const [micOn, setMicOn] = createSignal(true);
  const [camOn, setCamOn] = createSignal(true);
  const [selfActor, setSelfActor] = createSignal<ChatActor | null>(null);
  const [roster, setRoster] = createSignal<RtcParticipant[]>([]);
  const [remoteStreams, setRemoteStreams] = createSignal<
    Record<string, MediaStream>
  >({});

  let pc: RTCPeerConnection | null = null;

  const remoteTiles = createMemo<RemoteTile[]>(() => {
    const self = selfActor();
    const selfId = self ? streamIdOf(self) : null;
    const streams = remoteStreams();
    return roster()
      .filter((participant) => streamIdOf(participant.actor) !== selfId)
      .map((participant) => {
        const streamId = streamIdOf(participant.actor);
        return { streamId, participant, stream: streams[streamId] ?? null };
      });
  });

  const participantCount = createMemo(() => roster().length);

  const sendRtc = (signal: RtcClientSignal): boolean => {
    if (socket.isBinary()) {
      return socket.send(encodeRtcClientFrame(signal));
    }
    return socket.send(JSON.stringify({ type: "rtc", ...signal }));
  };

  const resetCall = () => {
    if (pc) {
      pc.onicecandidate = null;
      pc.ontrack = null;
      pc.onconnectionstatechange = null;
      pc.close();
      pc = null;
    }
    const stream = localStream();
    if (stream) {
      for (const track of stream.getTracks()) track.stop();
    }
    setLocalStream(null);
    setRoster([]);
    setRemoteStreams({});
    setMicOn(true);
    setCamOn(true);
    setCallState("idle");
  };

  const upsertParticipant = (participant: RtcParticipant) => {
    const key = streamIdOf(participant.actor);
    setRoster((prev) => {
      const without = prev.filter((item) => streamIdOf(item.actor) !== key);
      return [...without, participant];
    });
  };

  const removeParticipant = (actor: ChatActor) => {
    const key = streamIdOf(actor);
    setRoster((prev) => prev.filter((item) => streamIdOf(item.actor) !== key));
    setRemoteStreams((prev) => {
      if (!(key in prev)) return prev;
      const next = { ...prev };
      delete next[key];
      return next;
    });
  };

  const handleRtcSignal = async (signal: RtcServerSignal) => {
    const peer = pc;
    switch (signal.kind) {
      case "answer":
        if (!peer) return;
        try {
          await peer.setRemoteDescription({ type: "answer", sdp: signal.sdp });
          setCallState("in_call");
        } catch (err) {
          console.error("Failed to apply SFU answer:", err);
        }
        break;
      case "offer":
        if (!peer) return;
        try {
          await peer.setRemoteDescription({ type: "offer", sdp: signal.sdp });
          const answer = await peer.createAnswer();
          await peer.setLocalDescription(answer);
          sendRtc({ kind: "answer", sdp: answer.sdp ?? "" });
        } catch (err) {
          console.error("Failed to answer SFU renegotiation offer:", err);
        }
        break;
      case "ice":
        if (!peer) return;
        try {
          await peer.addIceCandidate(signalToIceInit(signal));
        } catch (err) {
          console.error("Failed to add remote ICE candidate:", err);
        }
        break;
      case "peer_state":
        if (signal.phase === "left") {
          removeParticipant(signal.actor);
        } else {
          upsertParticipant({
            actor: signal.actor,
            mic_on: signal.mic_on,
            cam_on: signal.cam_on,
          });
        }
        break;
      case "roster":
        setRoster(signal.participants);
        break;
      case "error":
        setCallError(signal.message);
        if (FATAL_RTC_ERROR_CODES.has(signal.code)) {
          resetCall();
        }
        break;
    }
  };

  const joinCall = async () => {
    if (callState() !== "idle" && callState() !== "error") return;
    setCallError(null);
    setCallState("joining");

    let stream: MediaStream;
    try {
      stream = await requestUserMedia();
    } catch (err) {
      console.error("getUserMedia failed:", err);
      setCallError("Microphone/camera permission denied.");
      setCallState("error");
      return;
    }
    setLocalStream(stream);
    setMicOn(true);
    setCamOn(true);

    const peer = createPeerConnection();
    pc = peer;
    peer.onicecandidate = (event) => {
      if (event.candidate) {
        sendRtc(iceCandidateToSignal(event.candidate));
      }
    };
    peer.ontrack = (event) => {
      const stream = event.streams[0];
      if (!stream) return;
      setRemoteStreams((prev) => ({ ...prev, [stream.id]: stream }));
    };
    peer.onconnectionstatechange = () => {
      if (peer.connectionState === "failed") {
        setCallError("Call connection failed.");
      }
    };

    for (const track of stream.getTracks()) {
      peer.addTrack(track, stream);
    }
    applyCodecPreferences(peer);

    try {
      const offer = await peer.createOffer();
      await peer.setLocalDescription(offer);
      const sent = sendRtc({
        kind: "join",
        sdp: offer.sdp ?? "",
        want_audio: true,
        want_video: true,
      });
      if (!sent) {
        setCallError("Not connected.");
        resetCall();
      }
    } catch (err) {
      console.error("Failed to create call offer:", err);
      setCallError("Could not start the call.");
      resetCall();
    }
  };

  const leaveCall = () => {
    if (callState() === "idle") return;
    sendRtc({ kind: "leave" });
    resetCall();
  };

  const toggleMic = () => {
    const stream = localStream();
    if (!stream) return;
    const next = !micOn();
    for (const track of stream.getAudioTracks()) track.enabled = next;
    setMicOn(next);
    sendRtc({ kind: "media_state", mic_on: next, cam_on: camOn() });
  };

  const toggleCamera = () => {
    const stream = localStream();
    if (!stream) return;
    const next = !camOn();
    for (const track of stream.getVideoTracks()) track.enabled = next;
    setCamOn(next);
    sendRtc({ kind: "media_state", mic_on: micOn(), cam_on: next });
  };

  const dispose = socket.onEvent((event) => {
    if (event.type === "hello") {
      setSelfActor(event.actor);
      return;
    }
    if (event.type === "rtc") {
      void handleRtcSignal(event);
    }
  });
  onCleanup(dispose);

  // If the socket drops while in a call, the SFU has torn the peer down; reset
  // local call state so the UI returns to idle.
  createEffect(
    () => ({ state: socket.connectionState(), inCall: callState() !== "idle" }),
    ({ state, inCall }) => {
      if ((state === "closed" || state === "error") && inCall) {
        resetCall();
      }
    },
  );

  onCleanup(() => {
    if (callState() !== "idle") resetCall();
  });

  const value: RtcContextValue = {
    callState,
    callError,
    localStream,
    micOn,
    camOn,
    selfActor,
    remoteTiles,
    participantCount,
    joinCall,
    leaveCall,
    toggleMic,
    toggleCamera,
  };

  return <RtcContext value={value}>{props.children}</RtcContext>;
};

export function useRtc(): RtcContextValue {
  // Throws ContextNotFoundError when used outside RtcProvider.
  return useContext(RtcContext);
}
