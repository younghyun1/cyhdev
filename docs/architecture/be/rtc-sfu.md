# Live Chat RTC: in-process SFU design

The audio/video layer is a Selective Forwarding Unit running inside the Axum binary on top of `rtc`/`webrtc` 0.21. Each browser holds one `RTCPeerConnection` to the SFU: it publishes its microphone and camera and subscribes to every other participant's tracks. Media uses DTLS-SRTP between each browser and the SFU; the SFU decrypts and re-encrypts transport packets while forwarding encoded media without transcoding.

Runtime ownership follows the backend feature boundary: persistence-independent signaling values live in `features/live_chat/domain/rtc.rs`; JSON signal serialization and the binary codec live in `features/live_chat/api`; the bounded chat cache and WebRTC engine, peer, publication, room, and coordinator live in `features/live_chat/service`.

## Topology
One room (`room_key = "main"` today; the registry is keyed by `room_key` to allow more). A room is an `RtcRoom` holding a registry of `RtcPeer` keyed by the WS `connection_id`. The room is created on the first join (which opens a `live_chat_calls` row) and removed when the last peer leaves (which closes the row). State is bounded: no idle rooms persist.

## Forwarding model
Each publication has a bounded 512-packet RTP broadcast channel and creates a distinct `TrackLocalStaticRTP` for each subscriber; the library binds each local track to one connection. Stream IDs remain actor-based to group audio and video in the browser. Forwarding starts after the subscriber answers negotiation. Cost is O(publishers x subscribers) packet copies; participants are capped by `RTC_MAX_PARTICIPANTS` and each subscription map has a hard 128-track bound.

The final interceptor retains only PLI/FIR keyframe requests and marks them `DeliverToApplication`, as required by RTC 0.21. Default interceptors process reports and transport feedback first. The SFU relays keyframe requests upstream with a per-publication 500 ms throttle. Teardown closes publication tasks explicitly, removes subscriber senders and deduplication entries, then renegotiates; ordinary leave/rejoin and WebSocket reconnect must both restore fresh media.

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

Per-peer renegotiation is coalesced by the peer's negotiation state to prevent overlapping offers, and an answer that arrives with no outstanding SFU offer is ignored. Mute/camera toggles never renegotiate; they only flip `track.enabled` client-side and emit MediaState for other clients' UI.

No participant can stall another. Unicast signals are queued with `try_send`; a full 64-signal queue means the client stopped reading, so that peer is torn down in memory (as on connection failure) and its participant row closes when the socket ends or the client rejoins. Fan-out and teardown renegotiate each subscriber in its own task, so the departing peer's slot release and Left broadcast never wait on other peers, and every socket write is bounded by the connection writer's 10-second send timeout.

## ICE / network
Each peer connection owns a UDP socket allocated from the bounded range starting at `RTC_UDP_PORT_START`, with `RTC_MAX_PARTICIPANTS` ports. `SettingEngineBuilder::with_nat_1to1_ips([RTC_PUBLIC_IP], Host).build()` configures the public host candidate using the rtc/webrtc 0.21 builder API. Deployment must expose the configured UDP range. External STUN/TURN is unnecessary for a public-IP server; optional `RTC_TURN_*` can be configured as a relay fallback for symmetric-NAT clients.

Browser-supplied candidates pass `service/rtc/ice_policy.rs` before reaching the ICE agent, both when trickled and as `a=candidate` lines inside offers and answers: at most 32 per peer, literal IP addresses only, and only globally routable ones unless `RTC_PUBLIC_IP` is itself loopback or private (local development). mDNS `.local` names are always dropped. This does not affect connectivity: browsers have no ICE servers and gather host candidates only, while the browser, as the controlling agent, sends checks to the SFU's advertised host candidate and the SFU learns the NAT-mapped address as a peer-reflexive candidate from those authenticated checks. Unfiltered candidates would only make the SFU probe its own network or multicast groups, and resolving mDNS names would query the server's LAN.

## Persistence
`live_chat_calls` (`live_chat_call_id` UUIDv7 PK, `room_key`, `call_started_at`, `call_ended_at`) and `live_chat_call_participants` (`live_chat_call_participant_id` PK, `live_chat_call_id` FK, nullable `user_id`/`guest_ip`, `participant_sender_kind`, `participant_display_name`, join/leave timestamps, `participant_had_audio`/`participant_had_video`) mirror the `live_chat_messages` conventions: table-prefixed columns, identity CHECK, indexed ID/sortable columns. No media is recorded.

## Safety
Bans (`live_chat_bans`) are checked at join and reject RTC the same way they reject chat. The participant cap rejects overflow. The connection inherits the existing auth/ban/rate middleware on the WS upgrade.
