# Live Chat RTC: call UI and state

The call feature reuses the existing live-chat WebSocket for signaling and adds a WebRTC peer connection plus a video grid to the chat panel.

## Shared socket
`LiveChatPanel` previously owned the WebSocket locally. The connection is lifted into a `LiveChatSocketContext` that owns the single `/ws/live-chat` socket, exposes `send(frame)` and a subscribe API, and is consumed by both `LiveChatPanel` (chat) and `RtcContext` (calls). One socket carries both chat and signaling frames; the binary protocol distinguishes them by opcode.

## RtcContext (`src/state/rtc.tsx`)
Global SolidJS context owning call state: the `RTCPeerConnection`, the local `MediaStream` from `getUserMedia` (audio + video), a map of remote streams keyed by actor key, the roster, the call connection state, and the local mic/camera flags. Actions: `joinCall()`, `leaveCall()`, `toggleMic()`, `toggleCamera()`.

Negotiation is perfect-negotiation-lite: the client offers once on join; afterward it only answers SFU offers. ICE candidates are trickled both ways. Remote tracks are routed to the right tile via `event.streams[0].id` (the SFU sets `stream_id` to the publisher's actor key). Mute and camera toggles flip `track.enabled` locally and send a MediaState signal; they do not renegotiate.

## Services
- `src/services/rtc.ts`: builds the `RTCPeerConnection`, orders codec preferences (AV1 > H265 > VP9 > VP8, Opus for audio) via `setCodecPreferences` where supported, requests media with sane constraints, and serializes/parses ICE candidates to the signal shapes.
- `src/services/live_chat_binary.ts`: extended with the RTC opcodes, encoders, and decoders, kept symmetric with the Rust codec.

## Components (`src/components/call/`)
- `CallPanel`: the video grid plus control bar, rendered above the message list in `LiveChatPanel` "full" mode when a call is active; a compact "call active (N)" pill otherwise.
- `VideoTile`: one participant; a `<video>` bound to the remote `MediaStream`, falling back to the `UserBadge` profile picture when the camera is off, with mic-muted and speaking indicators.
- `CallControls`: join/leave, mute, camera toggle.

## Styling
All call styles live in `src/styles/pageStyles.ts` (the project's single style source, Tailwind semantic strings), consistent with the rest of the app.

## DTOs and tests
RTC signal shapes for the JSON fallback live under `src/dtos/requests|responses/live_chat/rtc.ts`. Vitest covers the new binary opcode roundtrips (extending `src/__tests__/live_chat_binary.test.ts`) and the codec-preference ordering.

See `docs/architecture/be/rtc-sfu.md` for the wire protocol.
