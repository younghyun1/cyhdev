//! Per-connection token bucket covering every client event.
//!
//! Chat messages already have a per-address abuse limit, but typing,
//! heartbeat, media-state, and call signals were unmetered, and each could
//! trigger room-wide work. Every decoded event now spends tokens by cost; an
//! empty bucket drops the event, and a client that keeps sending into an
//! empty bucket is disconnected.

use std::time::Duration;

use tokio::time::Instant;

use super::protocol::DecodedLiveChatClientEvent;
use crate::features::live_chat::domain::rtc::RtcClientSignal;

/// Tokens are tracked in thousandths so refill needs no floating point.
const MILLI: u32 = 1_000;
/// Burst allowance: a call join with a full trickle of ICE candidates fits.
const BUCKET_CAPACITY_TOKENS: u32 = 32;
/// Sustained rate, far above human typing and chatting.
const REFILL_TOKENS_PER_SECOND: u32 = 8;
/// Rejected events tolerated back to back before the connection closes.
const MAX_CONSECUTIVE_REJECTIONS: u16 = 64;
/// Spacing between `rate_limited` notices so rejections cannot amplify output.
const REJECTION_NOTICE_INTERVAL: Duration = Duration::from_secs(2);

/// Cost of one Ping control frame; tungstenite answers each with a Pong.
pub(super) const PING_FRAME_COST: u32 = 1;

/// Token cost by event kind, weighted by the server work each one causes.
pub(super) fn event_cost(event: &DecodedLiveChatClientEvent) -> u32 {
    match event {
        // Persistence plus a room broadcast.
        DecodedLiveChatClientEvent::SendMessage { .. } => 2,
        DecodedLiveChatClientEvent::Typing { .. }
        | DecodedLiveChatClientEvent::Heartbeat { .. } => 1,
        DecodedLiveChatClientEvent::Rtc(signal) => match signal {
            // Peer connection, SDP negotiation, a database row, and fan-out.
            RtcClientSignal::Join { .. } => 8,
            // Teardown and renegotiation of every other participant.
            RtcClientSignal::Leave | RtcClientSignal::Answer { .. } => 2,
            RtcClientSignal::Ice(_) | RtcClientSignal::MediaState { .. } => 1,
        },
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum BudgetDecision {
    Admit,
    /// Drop the event; `notify` asks the caller to send a `rate_limited` error.
    Reject {
        notify: bool,
    },
    Close,
}

pub(super) struct ClientEventBudget {
    milli_tokens: u32,
    refilled_at: Instant,
    consecutive_rejections: u16,
    last_notice_at: Option<Instant>,
}

impl ClientEventBudget {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            milli_tokens: BUCKET_CAPACITY_TOKENS * MILLI,
            refilled_at: now,
            consecutive_rejections: 0,
            last_notice_at: None,
        }
    }

    pub(super) fn spend(&mut self, cost_tokens: u32, now: Instant) -> BudgetDecision {
        self.refill(now);
        let cost = cost_tokens.saturating_mul(MILLI);
        if self.milli_tokens >= cost {
            self.milli_tokens -= cost;
            self.consecutive_rejections = 0;
            return BudgetDecision::Admit;
        }
        self.consecutive_rejections = self.consecutive_rejections.saturating_add(1);
        if self.consecutive_rejections >= MAX_CONSECUTIVE_REJECTIONS {
            return BudgetDecision::Close;
        }
        let notify = match self.last_notice_at {
            Some(last) => now.saturating_duration_since(last) >= REJECTION_NOTICE_INTERVAL,
            None => true,
        };
        if notify {
            self.last_notice_at = Some(now);
        }
        BudgetDecision::Reject { notify }
    }

    fn refill(&mut self, now: Instant) {
        let elapsed_millis = now.saturating_duration_since(self.refilled_at).as_millis();
        // Tokens per second equal milli-tokens per millisecond.
        let earned = elapsed_millis.saturating_mul(u128::from(REFILL_TOKENS_PER_SECOND));
        let refilled = u128::from(self.milli_tokens).saturating_add(earned);
        self.milli_tokens = u32::try_from(refilled.min(u128::from(BUCKET_CAPACITY_TOKENS * MILLI)))
            .unwrap_or(BUCKET_CAPACITY_TOKENS * MILLI);
        self.refilled_at = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn burst_is_bounded_and_refills_over_time() {
        let start = Instant::now();
        let mut budget = ClientEventBudget::new(start);
        for _ in 0..BUCKET_CAPACITY_TOKENS {
            assert_eq!(budget.spend(1, start), BudgetDecision::Admit);
        }
        assert_eq!(
            budget.spend(1, start),
            BudgetDecision::Reject { notify: true }
        );
        assert_eq!(
            budget.spend(1, start),
            BudgetDecision::Reject { notify: false }
        );
        let later = start + Duration::from_millis(250);
        assert_eq!(budget.spend(2, later), BudgetDecision::Admit);
    }

    #[test]
    fn sustained_flooding_closes_the_connection() {
        let start = Instant::now();
        let mut budget = ClientEventBudget::new(start);
        assert_eq!(
            budget.spend(BUCKET_CAPACITY_TOKENS, start),
            BudgetDecision::Admit
        );
        let mut decision = BudgetDecision::Admit;
        for _ in 0..MAX_CONSECUTIVE_REJECTIONS {
            decision = budget.spend(1, start);
        }
        assert_eq!(decision, BudgetDecision::Close);
    }

    #[test]
    fn rejection_notices_are_spaced() {
        let start = Instant::now();
        let mut budget = ClientEventBudget::new(start);
        let _ = budget.spend(BUCKET_CAPACITY_TOKENS, start);
        assert_eq!(
            budget.spend(8, start),
            BudgetDecision::Reject { notify: true }
        );
        let soon = start + Duration::from_millis(500);
        assert_eq!(
            budget.spend(8, soon),
            BudgetDecision::Reject { notify: false }
        );
        let later = start + REJECTION_NOTICE_INTERVAL + Duration::from_millis(1);
        assert_eq!(
            budget.spend(64, later),
            BudgetDecision::Reject { notify: true }
        );
    }

    #[test]
    fn a_join_with_trickled_candidates_fits_the_burst() {
        let start = Instant::now();
        let mut budget = ClientEventBudget::new(start);
        let join = DecodedLiveChatClientEvent::Rtc(RtcClientSignal::Join {
            sdp: String::new(),
            want_audio: true,
            want_video: true,
        });
        assert_eq!(
            budget.spend(event_cost(&join), start),
            BudgetDecision::Admit
        );
        for _ in 0..20 {
            let ice = DecodedLiveChatClientEvent::Rtc(RtcClientSignal::Ice(
                crate::features::live_chat::domain::rtc::RtcIceCandidate {
                    candidate: String::new(),
                    sdp_mid: None,
                    sdp_mline_index: None,
                },
            ));
            assert_eq!(budget.spend(event_cost(&ice), start), BudgetDecision::Admit);
        }
    }
}
