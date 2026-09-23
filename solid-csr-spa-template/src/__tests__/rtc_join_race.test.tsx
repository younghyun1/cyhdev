import { cleanup, render } from "@solidjs/testing-library";
import { flush } from "solid-js";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type * as RtcServices from "../services/rtc";
import { RtcProvider, useRtc } from "../state/rtc";

interface FakeTrack {
  kind: "audio" | "video";
  enabled: boolean;
  stop: ReturnType<typeof vi.fn>;
}

interface FakePeer {
  close: ReturnType<typeof vi.fn>;
  resolveOffer: (() => void) | null;
}

const harness = vi.hoisted(() => ({
  frames: [] as string[],
  setConnectionState: null as ((state: "open" | "closed") => void) | null,
  resolveMedia: null as ((stream: MediaStream) => void) | null,
  deferOffer: false,
  peers: [] as FakePeer[],
}));

vi.mock("../state/live_chat_socket", async () => {
  const { createSignal } = await import("solid-js");
  const [connectionState, setConnectionState] = createSignal<"open" | "closed">("open");
  harness.setConnectionState = (state) => setConnectionState(state);
  const socket = {
    connectionState,
    isBinary: () => false,
    send: (frame: ArrayBuffer | string) => {
      harness.frames.push(typeof frame === "string" ? frame : "binary");
      return true;
    },
    acquire: () => () => undefined,
    onEvent: () => () => undefined,
  };
  return { useLiveChatSocket: () => socket };
});

vi.mock("../services/rtc", async (importOriginal) => {
  const actual = await importOriginal<typeof RtcServices>();
  return {
    ...actual,
    applyCodecPreferences: () => undefined,
    requestUserMedia: () =>
      new Promise<MediaStream>((resolve) => {
        harness.resolveMedia = resolve;
      }),
    createPeerConnection: () => {
      const peer: FakePeer = { close: vi.fn(), resolveOffer: null };
      harness.peers.push(peer);
      const connection = {
        onicecandidate: null,
        ontrack: null,
        onconnectionstatechange: null,
        connectionState: "new",
        close: peer.close,
        addTrack: () => undefined,
        getTransceivers: () => [],
        createOffer: () =>
          new Promise<RTCSessionDescriptionInit>((resolve) => {
            const offer = { type: "offer" as const, sdp: "v=0" };
            if (!harness.deferOffer) return resolve(offer);
            peer.resolveOffer = () => resolve(offer);
          }),
        setLocalDescription: () => Promise.resolve(),
      };
      return connection as unknown as RTCPeerConnection;
    },
  };
});

function fakeStream(): { stream: MediaStream; tracks: FakeTrack[] } {
  const tracks: FakeTrack[] = [
    { kind: "audio", enabled: true, stop: vi.fn() },
    { kind: "video", enabled: true, stop: vi.fn() },
  ];
  const stream = {
    getTracks: () => tracks,
    getAudioTracks: () => tracks.filter((track) => track.kind === "audio"),
    getVideoTracks: () => tracks.filter((track) => track.kind === "video"),
  };
  return { stream: stream as unknown as MediaStream, tracks };
}

function renderRtc(): ReturnType<typeof useRtc> {
  let context: ReturnType<typeof useRtc> | undefined;
  const Probe = () => {
    context = useRtc();
    return null;
  };
  render(() => (
    <RtcProvider>
      <Probe />
    </RtcProvider>
  ));
  if (!context) throw new Error("RTC context was not provided");
  return context;
}

const joinFrames = () => harness.frames.filter((frame) => frame.includes('"join"'));

describe("call join race", () => {
  beforeEach(() => {
    harness.frames = [];
    harness.peers = [];
    harness.resolveMedia = null;
    harness.deferOffer = false;
    harness.setConnectionState?.("open");
    flush();
  });
  afterEach(() => cleanup());

  it("releases devices and never joins when Leave wins the camera prompt", async () => {
    const rtc = renderRtc();
    // Solid 2 applies signal writes at flush; a real click is a later task.
    const joining = rtc.joinCall();
    flush();
    expect(rtc.callState()).toBe("joining");
    rtc.leaveCall();
    flush();
    expect(rtc.callState()).toBe("idle");

    const { stream, tracks } = fakeStream();
    harness.resolveMedia?.(stream);
    await joining;
    flush();

    for (const track of tracks) expect(track.stop).toHaveBeenCalled();
    expect(harness.peers).toHaveLength(0);
    expect(joinFrames()).toHaveLength(0);
    expect(rtc.localStream()).toBeNull();
    expect(rtc.callState()).toBe("idle");
  });

  it("abandons the offer when the socket drops mid-join", async () => {
    harness.deferOffer = true;
    const rtc = renderRtc();
    const joining = rtc.joinCall();
    flush();
    const { stream, tracks } = fakeStream();
    harness.resolveMedia?.(stream);
    await vi.waitFor(() => expect(harness.peers).toHaveLength(1));
    flush();

    harness.setConnectionState?.("closed");
    flush();
    expect(rtc.callState()).toBe("idle");

    harness.peers[0]?.resolveOffer?.();
    await joining;
    flush();

    expect(harness.peers[0]?.close).toHaveBeenCalled();
    for (const track of tracks) expect(track.stop).toHaveBeenCalled();
    expect(joinFrames()).toHaveLength(0);
    expect(rtc.localStream()).toBeNull();
  });

  it("still joins when nothing interrupts the attempt", async () => {
    const rtc = renderRtc();
    const joining = rtc.joinCall();
    flush();
    const { stream, tracks } = fakeStream();
    harness.resolveMedia?.(stream);
    await joining;
    flush();

    expect(joinFrames()).toHaveLength(1);
    for (const track of tracks) expect(track.stop).not.toHaveBeenCalled();
    rtc.leaveCall();
    flush();
    for (const track of tracks) expect(track.stop).toHaveBeenCalled();
  });
});
