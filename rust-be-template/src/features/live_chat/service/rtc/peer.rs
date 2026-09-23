//! A single participant's server-side peer connection in the SFU.
//!
//! Each `RtcPeer` owns one `RTCPeerConnection`, the local fan-out tracks built
//! from the media it publishes, and a per-peer renegotiation state machine. The
//! SFU is always the offerer for renegotiations (peer join/leave); the
//! coalescing `NegotiationState` prevents overlapping offers (glare).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock, Weak};

use tokio::sync::{Mutex, mpsc, watch};
use tracing::{debug, warn};
use uuid::Uuid;
use webrtc::peer_connection::{PeerConnection, RTCIceCandidateInit};
use webrtc::rtp_transceiver::RtpSender;

use super::ice_policy::{MAX_REMOTE_CANDIDATES_PER_PEER, RemoteCandidatePolicy};
use super::publication::RtcPublication;
use super::room::RtcRoom;
use crate::features::live_chat::domain::{
    actor::{ChatActor, ChatActorKey},
    rtc::{MediaKind, RtcIceCandidate, RtcParticipant, RtcServerSignal},
};

/// Driver event handling lives separately because `webrtc` 0.20 installs the
/// handler while constructing the peer connection.
mod events;
/// SDP/renegotiation methods live in the child module; they need access to this
/// type's private fields, which descendant modules are permitted.
mod negotiation;
/// Non-blocking unicast signal delivery.
mod signal_queue;
/// Subscription/keyframe methods live in a child module for the same reason.
mod subscription;

pub(crate) use events::RtcPeerEventHandler;
use negotiation::NegotiationState;
use signal_queue::{SignalPush, SignalQueue};

/// Stable per-publisher stream id so a browser groups a publisher's audio and
/// video into one `MediaStream` and the frontend can map it back to an actor.
/// Guests use their opaque keyed hash; SDP `msid` values are browser visible.
pub fn actor_stream_id(actor: &ChatActor) -> String {
    match &actor.actor_key {
        ChatActorKey::User(user_id) => format!("user:{user_id}"),
        ChatActorKey::Guest(guest_key) => format!("guest:{guest_key}"),
    }
}

/// Who a peer is: its WebSocket connection, public actor, and call row.
pub struct RtcPeerIdentity {
    pub connection_id: Uuid,
    pub actor: ChatActor,
    pub participant_id: Uuid,
}

/// One participant's peer connection and forwarding state.
pub struct RtcPeer {
    pub connection_id: Uuid,
    pub actor: ChatActor,
    pub participant_id: Uuid,
    pc: Arc<dyn PeerConnection>,
    signals: SignalQueue,
    /// Room that tears this peer down when its signal queue overflows.
    room: OnceLock<Weak<RtcRoom>>,
    /// This peer's published media, as fan-out publications others subscribe to.
    publications: scc::HashMap<MediaKind, Arc<RtcPublication>>,
    /// Track ids this peer is already subscribed to, so a fan-out racing the
    /// join-time subscribe cannot `add_track` the same source track twice.
    subscribed: Mutex<HashMap<String, Arc<dyn RtpSender>>>,
    /// Connection-local tracks waiting for the renegotiation answer that binds them.
    pending_subscriptions: Mutex<Vec<subscription::PendingSubscription>>,
    /// Cancels outgoing subscription tasks when this subscriber leaves.
    closed: watch::Sender<bool>,
    mic_on: AtomicBool,
    cam_on: AtomicBool,
    /// Which browser-supplied ICE candidates may reach the agent.
    candidate_policy: RemoteCandidatePolicy,
    /// Trickled candidates admitted so far, capped per peer.
    remote_candidates: AtomicUsize,
    negotiation: Mutex<NegotiationState>,
    /// Set once when teardown begins, so the Left broadcast and `pc.close()`
    /// happen exactly once across the WS-disconnect and connection-failed paths.
    torn_down: AtomicBool,
}

impl RtcPeer {
    /// Construct a peer wrapper. Handlers are attached separately so the
    /// callbacks can hold a `Weak` to the constructed `Arc<Self>`.
    pub fn new(
        identity: RtcPeerIdentity,
        pc: Arc<dyn PeerConnection>,
        signal_tx: mpsc::Sender<RtcServerSignal>,
        candidate_policy: RemoteCandidatePolicy,
        want_audio: bool,
        want_video: bool,
    ) -> Arc<Self> {
        Arc::new(Self {
            connection_id: identity.connection_id,
            actor: identity.actor,
            participant_id: identity.participant_id,
            pc,
            signals: SignalQueue::new(signal_tx),
            room: OnceLock::new(),
            publications: scc::HashMap::new(),
            subscribed: Mutex::new(HashMap::new()),
            pending_subscriptions: Mutex::new(Vec::new()),
            closed: watch::channel(false).0,
            mic_on: AtomicBool::new(want_audio),
            cam_on: AtomicBool::new(want_video),
            candidate_policy,
            remote_candidates: AtomicUsize::new(0),
            negotiation: Mutex::new(NegotiationState::default()),
            torn_down: AtomicBool::new(false),
        })
    }

    /// Attach the context used by the construction-time event handler.
    pub(crate) async fn attach_handlers(
        self: &Arc<Self>,
        handler: &RtcPeerEventHandler,
        room: Weak<RtcRoom>,
    ) {
        let _ = self.room.set(room.clone());
        handler.attach(Arc::downgrade(self), room).await;
    }

    /// Add a remote ICE candidate received from the client, subject to the
    /// address policy and the per-peer cap; see [`super::ice_policy`].
    pub async fn add_ice(&self, candidate: RtcIceCandidate) {
        if !self.candidate_policy.admits_candidate(&candidate.candidate) {
            debug!(connection_id = %self.connection_id, "Dropped inadmissible remote ICE candidate");
            return;
        }
        if self
            .remote_candidates
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |admitted| {
                (admitted < MAX_REMOTE_CANDIDATES_PER_PEER).then_some(admitted + 1)
            })
            .is_err()
        {
            debug!(connection_id = %self.connection_id, "Dropped remote ICE candidate above the per-peer cap");
            return;
        }
        let init = RTCIceCandidateInit {
            candidate: candidate.candidate,
            sdp_mid: candidate.sdp_mid,
            sdp_mline_index: candidate.sdp_mline_index,
            username_fragment: None,
            url: None,
        };
        if let Err(e) = self.pc.add_ice_candidate(init).await {
            debug!(error = %e, "add_ice_candidate failed");
        }
    }

    /// Record a microphone/camera state change (no renegotiation).
    pub fn set_media_state(&self, mic_on: bool, cam_on: bool) {
        self.mic_on.store(mic_on, Ordering::SeqCst);
        self.cam_on.store(cam_on, Ordering::SeqCst);
    }

    /// Current microphone-enabled flag.
    pub fn mic_on(&self) -> bool {
        self.mic_on.load(Ordering::SeqCst)
    }

    /// Current camera-enabled flag.
    pub fn cam_on(&self) -> bool {
        self.cam_on.load(Ordering::SeqCst)
    }

    /// Claim teardown for this peer. Returns true exactly once (the first caller
    /// across the WS-disconnect and connection-failed paths); later callers get
    /// false, so the Left broadcast and `pc.close()` run a single time.
    pub fn begin_teardown(&self) -> bool {
        self.torn_down
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    /// Whether teardown has started; a torn-down peer accepts no more signals.
    pub fn is_torn_down(&self) -> bool {
        self.torn_down.load(Ordering::SeqCst)
    }

    /// Roster entry for this peer.
    pub fn participant(&self) -> RtcParticipant {
        RtcParticipant {
            actor: self.actor.clone(),
            mic_on: self.mic_on(),
            cam_on: self.cam_on(),
        }
    }

    /// Queue a unicast signal to this peer's client without waiting. A full
    /// queue means the client stopped reading its socket; the peer is torn
    /// down instead of stalling room fan-out or teardown for everyone else.
    /// Returns whether the signal was queued.
    pub fn send_signal(&self, signal: RtcServerSignal) -> bool {
        match self.signals.push(signal) {
            SignalPush::Queued => true,
            SignalPush::Overloaded { first } => {
                if first {
                    self.tear_down_overloaded();
                }
                false
            }
            SignalPush::Closed => false,
        }
    }

    fn tear_down_overloaded(&self) {
        warn!(connection_id = %self.connection_id, "RTC signal queue full; tearing down slow peer");
        let Some(room) = self.room.get().and_then(Weak::upgrade) else {
            return;
        };
        let connection_id = self.connection_id;
        // Same in-memory teardown as a failed connection; the participant row
        // closes when the WebSocket ends or the client leaves.
        tokio::spawn(async move {
            room.handle_peer_dropped(connection_id).await;
        });
    }

    /// Close the underlying peer connection.
    pub async fn close(&self) {
        self.closed.send_replace(true);
        for publication in self.publications_snapshot().await {
            publication.close();
        }
        let mut subscribed = self.subscribed.lock().await;
        for (_, sender) in subscribed.drain() {
            if let Err(error) = self.pc.remove_track(&sender).await {
                debug!(%error, "Could not remove outgoing RTC sender during teardown");
            }
        }
        self.pending_subscriptions.lock().await.clear();
        drop(subscribed);
        if let Err(e) = self.pc.close().await {
            debug!(error = %e, "peer connection close failed");
        }
    }
}
