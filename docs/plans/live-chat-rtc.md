# Live Chat: Audio/Video Group Calls (in-process SFU)

## Context
The live chat is a single global broadcast room served over `/ws/live-chat` with a binary protocol (`livechat.bin.v1`), a `tokio::sync::broadcast` fan-out, and an `scc`-backed `LiveChatCache`, persisted to Postgres via Diesel. The SolidJS SPA is embedded in the single Axum binary via `rust-embed`, served over mandatory Rustls TLS (so the browser secure-context requirement for `getUserMedia`/WebRTC is already met). This feature adds group audio+video calling to the room while keeping media inside the one Rust binary.

Decisions: group room call (everyone in `main` shares one call); in-process SFU using the `webrtc` crate (webrtc-rs 0.17); audio and video together; clients encode (Opus audio, AV1/VP9/VP8/H264 video by capacity), the SFU forwards RTP without transcoding; call records persisted (no media recording).

## Constraints
- No transcoding. The SFU forwards opaque RTP, so all participants rendering a given publisher must share that publisher's video codec. AV1/VP9/VP8/Opus interoperate broadly; H265 in WebRTC is Safari-only and does not interop with Chrome, so it is best-effort. A browser that cannot decode the room codec will not render that publisher.
- SFU-initiated renegotiation. Initial join is a client offer; every subsequent renegotiation (peer join/leave adds/removes tracks) is SFU-offered. The SFU is the sole offerer post-join, which sidesteps glare. Mute/camera toggles are local track-enable flips and never renegotiate.
- Public-IP SFU. The server advertises its public IP as an ICE host candidate (`set_nat_1to1_ips`) on a single muxed UDP port, so clients dial it directly; no external TURN is needed in normal public deployments.

## Backend (rust-be-template)
- New domain module `src/domain/live_chat/rtc/`: `config.rs` (`RtcConfig::from_env`, nutype newtypes), `signal.rs` (client/server signal enums), `media_engine.rs` (MediaEngine + SettingEngine + UDP mux + nat_1to1_ips), `room.rs` (`RtcRoom`, per-room peer registry + roster + broadcast), `peer.rs` (`RtcPeer`, one `RTCPeerConnection`), `publication.rs` (per-publisher `TrackLocalStaticRTP` fan-out + RTP read loop).
- Signaling rides the existing WS: `binary_codec.rs` gains opcode `CLIENT_RTC = 0x05` (sub-opcode framed) and `SERVER_RTC = 0x90`; `LiveChatServerEvent::Rtc(RtcServerSignal)`; `LiveChatClientEvent::Rtc` for the JSON fallback. Unicast signals (answer/offer/ice) go to one connection via the per-connection `out_tx`; peer-state/roster broadcast room-wide via `cache.broadcast`. The `LIVE_CHAT_MAX_FRAME_BYTES` cap is raised to 64 KiB to fit SDP.
- `src/handlers/live_chat/ws/rtc.rs` dispatches RTC client signals to the room/peer and runs a relay task that encodes per-peer `RtcServerSignal` onto `out_tx`. `ws.rs` wires join/dispatch and tears the peer down on disconnect.
- State: `ServerState` gains `rtc_rooms: scc::HashMap<String, Arc<RtcRoom>>` and `rtc_config: RtcConfig`, both built in `builder.rs`. Bounded: a room is removed when it empties.
- Persistence: migration `live_chat_calls` + `live_chat_call_participants` (UUIDv7 PKs, table-prefixed columns, sender_kind 0/1, identity CHECK). A call row opens on first join to an empty room and closes when it empties; participant rows record join/leave and audio/video flags.
- The `PRUNE_LIVE_CHAT_STATE` job also drops empty/stale rooms and closes dangling call rows.

## Config / env
`RTC_ENABLE`, `RTC_PUBLIC_IP` (required when enabled), `RTC_UDP_MUX_PORT`, `RTC_MAX_PARTICIPANTS`, optional `RTC_TURN_URL`/`RTC_TURN_USER`/`RTC_TURN_PASS`. Dockerfile exposes the UDP port.

## Frontend (solid-csr-spa-template)
- The single WS is lifted into a shared `LiveChatSocketContext` consumed by `LiveChatPanel` and a new `RtcContext` (`src/state/rtc.tsx`) that owns the `RTCPeerConnection`, local `MediaStream`, remote streams, roster, and the join/leave/mute/camera actions with perfect-negotiation handling.
- `src/services/rtc.ts` (codec preference ordering, getUserMedia, ICE serialization); `live_chat_binary.ts` gains the new opcodes. Components under `src/components/call/`: `CallPanel`, `VideoTile`, `CallControls`, integrated into `LiveChatPanel` "full" mode. Styling centralized in `styles/pageStyles.ts`.

## Verification
`cargo check`/`cargo clippy`/`cargo fmt` (dev only); `cargo test` for codec roundtrips; `npm test` for binary opcodes. Local: run with `RTC_ENABLE=true RTC_PUBLIC_IP=127.0.0.1 RTC_UDP_MUX_PORT=<port>`, open two tabs at `/live-chat`, join, confirm two-way audio+video, mute/camera toggle, roster updates, clean teardown; check `getStats` for AV1/Opus. Confirm `live_chat_calls` opens/closes and participant rows record leave.

See `docs/architecture/be/rtc-sfu.md` and `docs/design/fe/call-ui.md`.
