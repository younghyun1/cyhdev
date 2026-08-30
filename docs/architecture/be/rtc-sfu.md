# Live Chat RTC: in-process SFU design

The audio/video layer is a Selective Forwarding Unit running inside the Axum binary on top of webrtc-rs (`webrtc` 0.17). Each browser holds one `RTCPeerConnection` to the SFU: it publishes its microphone and camera and subscribes to every other participant's tracks. Media is DTLS-SRTP encrypted end to peer by webrtc-rs; the SFU forwards RTP without decrypting application media beyond what SRTP requires, and never transcodes.

Runtime ownership follows the backend feature boundary: persistence-independent signaling values live in `features/live_chat/domain/rtc.rs`; JSON signal serialization and the binary codec live in `features/live_chat/api`; the bounded chat cache and WebRTC engine, peer, publication, room, and coordinator live in `features/live_chat/service`.

## Topology
One room (`room_key = "main"` today; the registry is keyed by `room_key` to allow more). A room is an `RtcRoom` holding a registry of `RtcPeer` keyed by the WS `connection_id`. The room is created on the first join (which opens a `live_chat_calls` row) and removed when the last peer leaves (which closes the row). State is bounded: no idle rooms persist.

## Forwarding model
For each published track the SFU creates exactly one `TrackLocalStaticRTP` (same codec capability as the inbound `TrackRemote`, `stream_id` set to the publisher's actor key) and `add_track`s it to every other peer. A single RTP read loop copies packets from the `TrackRemote` into that local track, which webrtc-rs fans out to all bound senders. Cost is O(publishers x subscribers) packet copies; participants are capped by `RTC_MAX_PARTICIPANTS`. The SFU forwards subscriber PLI/keyframe requests upstream so a newly-subscribed peer gets a keyframe.

## Signaling
Signaling rides the existing `/ws/live-chat` socket; there is no separate route. Frames extend the binary protocol; the JSON fallback mirrors them via serde.

Client to server (`CLIENT_RTC = 0x05`, then a sub-opcode):

| sub-op | name | payload |
| --- | --- | --- |
| 0x01 | Join | u8 want_audio, u8 want_video, string sdp (offer) |
| 0x02 | Answer | string sdp |
| 0x03 | Ice | string candidate, optional string sdp_mid, optional u16 sdp_mline_index |
| 0x04 | Leave | (empty) |
| 0x05 | MediaState | u8 mic_on, u8 cam_on |

Server to client (`SERVER_RTC = 0x90`, then a sub-opcode):

| sub-op | name | payload | delivery |
| --- | --- | --- | --- |
| 0x01 | Answer | string sdp | unicast |
| 0x02 | Offer | string sdp | unicast |
| 0x03 | Ice | candidate, optional mid, optional mline | unicast |
| 0x04 | PeerState | actor, u8 phase (0 left / 1 joined), u8 mic_on, u8 cam_on | broadcast |
| 0x05 | Roster | u8 count, then [actor, u8 mic_on, u8 cam_on] | unicast on join |
| 0x06 | Error | string code, string message | unicast |

Unicast signals reach one connection through that connection's `out_tx` mpsc (the existing writer task drains it); the RTC service emits `RtcServerSignal` values on a per-peer channel and a relay task in `ws/rtc.rs` encodes and forwards them. Broadcast signals (PeerState, and Roster echoes) go through `cache.broadcast` as `LiveChatServerEvent::Rtc(...)`, so non-call clients also learn a call is active. `LIVE_CHAT_MAX_FRAME_BYTES` is raised to 64 KiB because SDP exceeds the old 2 KiB cap; chat messages remain capped at 300 chars separately.

## Negotiation flow
1. Client sends Join with an offer. The SFU creates the peer, registers `on_track`/`on_ice_candidate`, sets the remote offer, adds existing publishers' local tracks, creates and sets the answer, and replies Answer. It then sends the Roster and broadcasts PeerState(joined).
2. When the new peer's tracks arrive (`on_track`), the SFU builds their `TrackLocalStaticRTP`, adds it to every other peer, and renegotiates each by sending an SFU Offer; those peers reply Answer.
3. Leave (or disconnect) removes the peer's tracks from others, renegotiates them, closes the `RTCPeerConnection`, records the participant leave, and broadcasts PeerState(left). The SFU is the only offerer for steps 2 and 3, so there is no glare.

Per-peer renegotiation is serialized by a tokio Mutex on the peer to prevent overlapping offers. Mute/camera toggles never renegotiate; they only flip `track.enabled` client-side and emit MediaState for other clients' UI.

## ICE / network
`SettingEngine` uses a single `UDPMuxDefault` bound to `RTC_UDP_MUX_PORT` and `set_nat_1to1_ips([RTC_PUBLIC_IP], Host)`, so the SFU advertises its public IP as a host candidate and all media multiplexes onto one UDP port (one Docker `EXPOSE`). External STUN/TURN is unnecessary for a public-IP server; optional `RTC_TURN_*` can be configured as a relay fallback for symmetric-NAT clients.

## Persistence
`live_chat_calls` (`live_chat_call_id` UUIDv7 PK, `room_key`, `call_started_at`, `call_ended_at`) and `live_chat_call_participants` (`live_chat_call_participant_id` PK, `live_chat_call_id` FK, nullable `user_id`/`guest_ip`, `participant_sender_kind`, `participant_display_name`, join/leave timestamps, `participant_had_audio`/`participant_had_video`) mirror the `live_chat_messages` conventions: table-prefixed columns, identity CHECK, indexed ID/sortable columns. No media is recorded.

## Safety
Bans (`live_chat_bans`) are checked at join and reject RTC the same way they reject chat. The participant cap rejects overflow. The connection inherits the existing auth/ban/rate middleware on the WS upgrade.
