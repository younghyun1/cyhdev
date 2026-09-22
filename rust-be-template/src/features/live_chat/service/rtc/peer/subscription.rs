//! Subscription management for [`RtcPeer`]: binding other publishers' fan-out
//! tracks onto this peer's connection and requesting the keyframes that make
//! subscribed video decodable.
//!
//! Split out of `peer.rs` to keep that file under the size limit. As a
//! descendant of `rtc::peer` this module may access `RtcPeer`'s private fields.

use std::sync::Arc;

use rtc::rtp::Packet;
use tokio::sync::broadcast;
use tracing::warn;
use webrtc::media_stream::track_local::TrackLocal;
use webrtc::media_stream::track_local::static_rtp::TrackLocalStaticRTP;

use super::super::publication::{RtcPublication, spawn_rtp_forward};
use super::RtcPeer;

// The configured room admits at most 64 participants, each publishing audio and video.
const MAX_SUBSCRIPTIONS: usize = 128;

/// A subscriber-local track waiting for its negotiation answer before forwarding starts.
pub(super) struct PendingSubscription {
    publication: Arc<RtcPublication>,
    local: Arc<TrackLocalStaticRTP>,
    packets: broadcast::Receiver<Packet>,
}

impl RtcPeer {
    /// Subscribe this peer to a publication. Returns true if the track was
    /// newly added (the caller should then renegotiate this peer). Idempotent
    /// per source track id: a fan-out racing the join-time subscribe cannot
    /// `add_track` the same track twice onto this peer connection.
    pub async fn subscribe_to(&self, publication: Arc<RtcPublication>) -> bool {
        let track_id = publication.track_id().to_owned();
        let mut subscribed = self.subscribed.lock().await;
        if *self.closed.borrow() || publication.is_closed() || subscribed.contains_key(&track_id) {
            return false;
        }
        if subscribed.len() >= MAX_SUBSCRIPTIONS {
            warn!("RTC subscription limit reached");
            return false;
        }
        let (local, packets) = publication.subscribe();
        match self
            .pc
            .add_track(local.clone() as Arc<dyn TrackLocal>)
            .await
        {
            Ok(sender) => {
                subscribed.insert(track_id, sender);
                self.pending_subscriptions
                    .lock()
                    .await
                    .push(PendingSubscription {
                        publication,
                        local,
                        packets,
                    });
                true
            }
            Err(e) => {
                warn!(error = %e, "add_track (subscribe) failed");
                false
            }
        }
    }

    /// Remove departed publications so the same actor can publish fresh tracks on rejoin.
    pub async fn unsubscribe_from(&self, publications: &[Arc<RtcPublication>]) -> bool {
        let mut subscribed = self.subscribed.lock().await;
        let mut changed = false;
        for publication in publications {
            let track_id = publication.track_id();
            if let Some(sender) = subscribed.remove(track_id) {
                if let Err(error) = self.pc.remove_track(&sender).await {
                    warn!(%error, "Could not remove departed RTC track");
                }
                changed = true;
            }
        }
        self.pending_subscriptions
            .lock()
            .await
            .retain(|pending| !pending.publication.is_closed());
        changed
    }

    /// Snapshot of this peer's publications.
    pub async fn publications_snapshot(&self) -> Vec<Arc<RtcPublication>> {
        let mut publications = Vec::new();
        self.publications
            .iter_async(|_, publication| {
                publications.push(publication.clone());
                true
            })
            .await;
        publications
    }

    /// Start forwarding for tracks bound by the latest negotiation answer.
    pub(super) async fn activate_pending_subscriptions(&self) {
        let pending = std::mem::take(&mut *self.pending_subscriptions.lock().await);
        for subscription in pending {
            spawn_rtp_forward(
                subscription.packets,
                subscription.local,
                subscription.publication.clone(),
                self.closed.subscribe(),
            );
            subscription.publication.request_keyframe().await;
        }
    }
}
