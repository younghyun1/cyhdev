//! Per-connection RTC signaling: dispatch client signals to the room/peer and
//! relay the peer's outbound signals onto the connection's writer queue.
//!
//! The SFU mechanics live in `features::live_chat::service::rtc`; this module is the glue
//! between a live-chat WebSocket connection and that peer.

use std::net::IpAddr;
use std::sync::Arc;

use tokio::sync::{Mutex, mpsc};
use tracing::error;
use uuid::Uuid;

use crate::features::live_chat::{
    domain::{
        actor::ChatActor,
        message::DEFAULT_LIVE_CHAT_ROOM,
        rtc::{RtcClientSignal, RtcPeerPhase, RtcServerSignal},
    },
    service::{
        live_chat_service::LiveChatService,
        rtc::{
            peer::{RtcPeer, RtcPeerEventHandler},
            room::{RtcRoom, RtcRoomAcquire},
        },
    },
};

use super::LiveChatWireProtocol;
use super::protocol::OutboundSender;

/// Bound on the per-peer outbound signal channel (SDP/ICE to one client).
const RTC_SIGNAL_QUEUE: usize = 64;

/// Mutable per-connection RTC state, guarded by a mutex on [`RtcSession`].
#[derive(Default)]
pub(super) struct RtcSessionInner {
    pub(super) room: Option<Arc<RtcRoom>>,
    pub(super) peer: Option<Arc<RtcPeer>>,
    pub(super) participant_id: Option<Uuid>,
}

/// RTC state bound to a single live-chat WebSocket connection.
pub(super) struct RtcSession {
    pub(super) service: Arc<LiveChatService>,
    pub(super) connection_id: Uuid,
    pub(super) actor: ChatActor,
    pub(super) client_ip: IpAddr,
    pub(super) out: OutboundSender,
    pub(super) wire_protocol: LiveChatWireProtocol,
    pub(super) inner: Mutex<RtcSessionInner>,
}

impl RtcSession {
    pub(super) fn new(
        service: Arc<LiveChatService>,
        connection_id: Uuid,
        actor: ChatActor,
        client_ip: IpAddr,
        out: OutboundSender,
        wire_protocol: LiveChatWireProtocol,
    ) -> Self {
        Self {
            service,
            connection_id,
            actor,
            client_ip,
            out,
            wire_protocol,
            inner: Mutex::new(RtcSessionInner::default()),
        }
    }

    /// Route one inbound client signal.
    pub(super) async fn dispatch(&self, signal: RtcClientSignal) {
        if let Some(user_id) = self.actor.user_id
            && self.service.cache.is_connected_user_disabled(user_id)
        {
            self.leave().await;
            self.send_error(
                "account_deleted",
                "This account can no longer use live chat calls.",
            )
            .await;
            return;
        }
        match signal {
            RtcClientSignal::Join {
                sdp,
                want_audio,
                want_video,
            } => self.handle_join(sdp, want_audio, want_video).await,
            RtcClientSignal::Answer { sdp } => {
                if let Some(peer) = self.current_peer().await {
                    peer.accept_answer(sdp).await;
                }
            }
            RtcClientSignal::Ice(candidate) => {
                if let Some(peer) = self.current_peer().await {
                    peer.add_ice(candidate).await;
                }
            }
            RtcClientSignal::Leave => self.leave().await,
            RtcClientSignal::MediaState { mic_on, cam_on } => {
                let (room, peer) = self.current_room_peer().await;
                if let (Some(room), Some(peer)) = (room, peer) {
                    peer.set_media_state(mic_on, cam_on);
                    room.broadcast_peer_state(&peer.participant(), RtcPeerPhase::Joined);
                }
            }
        }
    }

    /// Authoritative teardown on WS disconnect: leave the call if joined.
    pub(super) async fn teardown(&self) {
        self.leave().await;
    }

    async fn handle_join(&self, sdp: String, want_audio: bool, want_video: bool) {
        if self.current_peer().await.is_some() {
            // Already in the call; ignore duplicate join.
            return;
        }

        let engine = match self.service.rtc.engine() {
            Some(engine) => engine,
            None => {
                self.send_error("rtc_disabled", "Calls are not available.")
                    .await;
                return;
            }
        };

        if self
            .service
            .is_actor_banned(self.actor.user_id, self.client_ip)
            .await
        {
            self.send_error("banned", "Live chat access denied.").await;
            return;
        }

        // Acquire reserves a participant slot atomically (enforces the cap and
        // keeps the room alive against concurrent GC). Every failure path below
        // must release_slot() so the reservation is not leaked.
        let room = match self
            .service
            .rtc
            .acquire_room(DEFAULT_LIVE_CHAT_ROOM, self.actor.user_id)
            .await
        {
            RtcRoomAcquire::Acquired(room) => room,
            RtcRoomAcquire::Full => {
                self.send_error("room_full", "The call is full.").await;
                return;
            }
            RtcRoomAcquire::Unavailable => {
                self.send_error("rtc_unavailable", "Call could not be started.")
                    .await;
                return;
            }
        };

        let peer_event_handler = RtcPeerEventHandler::new();
        let pc = match engine.new_peer_connection(peer_event_handler.clone()).await {
            Ok(pc) => pc,
            Err(e) => {
                error!(error = %e, "Failed to create RTC peer connection");
                self.send_error("rtc_unavailable", "Call could not be started.")
                    .await;
                room.release_slot();
                self.service.rtc.remove_room_if_empty(&room.room_key).await;
                return;
            }
        };

        let participant_id = match self
            .service
            .rtc
            .participant_join(room.call_id, &self.actor, want_audio, want_video)
            .await
        {
            Some(id) => id,
            None => {
                if let Err(e) = pc.close().await {
                    error!(error = %e, "Failed to close peer connection after persist failure");
                }
                self.send_error("persist_failed", "Call could not be started.")
                    .await;
                room.release_slot();
                self.service.rtc.remove_room_if_empty(&room.room_key).await;
                return;
            }
        };

        let (rtc_signal_tx, rtc_signal_rx) = mpsc::channel::<RtcServerSignal>(RTC_SIGNAL_QUEUE);
        self.spawn_signal_relay(rtc_signal_rx);

        let peer = RtcPeer::new(
            self.connection_id,
            self.actor.clone(),
            participant_id,
            pc,
            rtc_signal_tx,
            want_audio,
            want_video,
        );
        peer.attach_handlers(&peer_event_handler, Arc::downgrade(&room))
            .await;

        let answer = match peer.answer_join_offer(sdp).await {
            Some(answer) => answer,
            None => {
                peer.close().await;
                self.service.rtc.participant_leave(participant_id).await;
                self.send_error("sdp_failed", "Could not negotiate the call.")
                    .await;
                room.release_slot();
                self.service.rtc.remove_room_if_empty(&room.room_key).await;
                return;
            }
        };
        peer.send_signal(RtcServerSignal::Answer { sdp: answer })
            .await;

        room.register_peer(peer.clone()).await;
        {
            let mut inner = self.inner.lock().await;
            inner.room = Some(room.clone());
            inner.peer = Some(peer.clone());
            inner.participant_id = Some(participant_id);
        }

        let participants = room.roster().await;
        peer.send_signal(RtcServerSignal::Roster { participants })
            .await;
        room.broadcast_peer_state(&peer.participant(), RtcPeerPhase::Joined);

        // Deliver existing publishers to the newcomer (SFU-offered renegotiation).
        room.subscribe_new_peer(peer).await;
    }
}
