//! SDP exchange and the coalescing renegotiation state machine for [`RtcPeer`].
//!
//! Split out of `peer.rs` to keep that file under the size limit. As a descendant
//! of `rtc::peer` this module may access `RtcPeer`'s private fields.

use std::sync::Arc;
use std::time::Duration;

use tokio::time::Instant;
use tracing::{debug, error, warn};
use webrtc::peer_connection::RTCSessionDescription;

use super::RtcPeer;
use crate::features::live_chat::domain::rtc::RtcServerSignal;

/// How long an unanswered SFU offer stays authoritative. Past this the offer is
/// treated as stale and replaced on the next renegotiation, so a client that
/// never returns an answer (backgrounded tab, lost answer) cannot permanently
/// wedge the peer's negotiation.
const STALE_OFFER_TIMEOUT: Duration = Duration::from_secs(10);

/// Coalescing renegotiation state. `making_offer` is set while an SFU offer is
/// outstanding (awaiting the client's answer); a renegotiation requested in that
/// window sets `pending` and is replayed once the answer arrives. `offer_at`
/// timestamps the outstanding offer so a never-answered offer (backgrounded tab,
/// lost answer) goes stale and is replaced on the next renegotiation rather than
/// wedging the peer forever.
#[derive(Default)]
pub(super) struct NegotiationState {
    making_offer: bool,
    pending: bool,
    offer_at: Option<Instant>,
}

#[derive(Debug, Eq, PartialEq)]
enum OfferStart {
    Start,
    /// A fresh offer is outstanding; this request replays after its answer.
    Coalesced,
    /// The outstanding offer went unanswered too long and is replaced.
    ReplaceStale,
}

impl NegotiationState {
    fn begin_offer(&mut self, now: Instant) -> OfferStart {
        let start = if self.making_offer {
            let stale = self
                .offer_at
                .map(|sent_at| now.saturating_duration_since(sent_at) >= STALE_OFFER_TIMEOUT)
                .unwrap_or(true);
            if !stale {
                self.pending = true;
                return OfferStart::Coalesced;
            }
            OfferStart::ReplaceStale
        } else {
            OfferStart::Start
        };
        self.making_offer = true;
        self.offer_at = Some(now);
        start
    }

    /// Settle the outstanding offer; true when a coalesced request must replay.
    fn finish_offer(&mut self) -> bool {
        self.abandon_offer();
        std::mem::take(&mut self.pending)
    }

    fn abandon_offer(&mut self) {
        self.making_offer = false;
        self.offer_at = None;
    }
}

impl RtcPeer {
    /// Apply the client's join offer and produce the SFU answer SDP.
    pub async fn answer_join_offer(&self, offer_sdp: String) -> Option<String> {
        let offer_sdp = self
            .candidate_policy
            .sanitize_remote_sdp(&offer_sdp)
            .into_owned();
        let offer = match RTCSessionDescription::offer(offer_sdp) {
            Ok(offer) => offer,
            Err(e) => {
                error!(error = %e, "Invalid join offer SDP");
                return None;
            }
        };
        if let Err(e) = self.pc.set_remote_description(offer).await {
            error!(error = %e, "set_remote_description(offer) failed");
            return None;
        }
        let answer = match self.pc.create_answer(None).await {
            Ok(answer) => answer,
            Err(e) => {
                error!(error = %e, "create_answer failed");
                return None;
            }
        };
        if let Err(e) = self.pc.set_local_description(answer.clone()).await {
            error!(error = %e, "set_local_description(answer) failed");
            return None;
        }
        Some(answer.sdp)
    }

    /// Start (or coalesce) an SFU-initiated renegotiation offer. If an offer is
    /// already outstanding and still fresh, the request is marked pending and
    /// replayed when the answer arrives; if the outstanding offer has gone stale
    /// (never answered) it is replaced. The offer is queued without waiting.
    pub async fn renegotiate(self: &Arc<Self>) {
        // Spawned fan-out or teardown work can outlive this peer's teardown.
        if self.is_torn_down() {
            return;
        }
        match self.negotiation.lock().await.begin_offer(Instant::now()) {
            OfferStart::Coalesced => return,
            OfferStart::ReplaceStale => warn!(
                connection_id = %self.connection_id,
                "Replacing stale unanswered renegotiation offer"
            ),
            OfferStart::Start => {}
        }

        let offer = match self.pc.create_offer(None).await {
            Ok(offer) => offer,
            Err(e) => {
                error!(error = %e, "create_offer (renegotiation) failed");
                self.negotiation.lock().await.abandon_offer();
                return;
            }
        };
        if let Err(e) = self.pc.set_local_description(offer.clone()).await {
            error!(error = %e, "set_local_description (renegotiation offer) failed");
            self.negotiation.lock().await.abandon_offer();
            return;
        }
        if !self.send_signal(RtcServerSignal::Offer { sdp: offer.sdp }) {
            self.negotiation.lock().await.abandon_offer();
        }
    }

    /// Apply a client answer to an SFU renegotiation offer, then replay a
    /// pending renegotiation if one was requested while the offer was in flight.
    /// An answer with no outstanding offer is ignored: applying it would fail
    /// or, worse, replay pending work the client never asked for.
    pub async fn accept_answer(self: &Arc<Self>, sdp: String) {
        if !self.negotiation.lock().await.making_offer {
            debug!(connection_id = %self.connection_id, "Ignored RTC answer without an outstanding offer");
            return;
        }
        let sdp = self.candidate_policy.sanitize_remote_sdp(&sdp).into_owned();
        match RTCSessionDescription::answer(sdp) {
            Ok(answer) => {
                if let Err(e) = self.pc.set_remote_description(answer).await {
                    error!(error = %e, "set_remote_description(answer) failed");
                } else {
                    // Subscriptions added in this negotiation round are live
                    // now; ask their publishers for the keyframes that make
                    // the video decodable.
                    self.activate_pending_subscriptions().await;
                }
            }
            Err(e) => error!(error = %e, "Invalid renegotiation answer SDP"),
        }

        let pending = self.negotiation.lock().await.finish_offer();
        if pending {
            self.renegotiate().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_offers_coalesce_and_replay_once() {
        let now = Instant::now();
        let mut state = NegotiationState::default();
        assert_eq!(state.begin_offer(now), OfferStart::Start);
        assert_eq!(state.begin_offer(now), OfferStart::Coalesced);
        assert_eq!(state.begin_offer(now), OfferStart::Coalesced);
        assert!(state.finish_offer());
        assert!(!state.making_offer);
        assert!(!state.finish_offer());
    }

    #[test]
    fn unanswered_offers_go_stale() {
        let now = Instant::now();
        let mut state = NegotiationState::default();
        assert_eq!(state.begin_offer(now), OfferStart::Start);
        let later = now + STALE_OFFER_TIMEOUT;
        assert_eq!(state.begin_offer(later), OfferStart::ReplaceStale);
        assert_eq!(state.offer_at, Some(later));
    }

    #[test]
    fn abandoned_offers_leave_no_outstanding_offer() {
        let now = Instant::now();
        let mut state = NegotiationState::default();
        assert_eq!(state.begin_offer(now), OfferStart::Start);
        state.abandon_offer();
        assert!(!state.making_offer);
        assert_eq!(state.begin_offer(now), OfferStart::Start);
    }
}
