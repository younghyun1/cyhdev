//! Non-blocking signal delivery to one peer's WebSocket relay.

use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::mpsc::{self, error::TrySendError};

use crate::features::live_chat::domain::rtc::RtcServerSignal;

/// Result of queueing one signal.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum SignalPush {
    Queued,
    /// The queue is full: the client stopped draining its socket. `first` is
    /// true only for the push that detected it, so teardown starts once.
    Overloaded {
        first: bool,
    },
    /// The relay is gone; the connection is already closing.
    Closed,
}

/// Bounded queue that never waits. Room fan-out and teardown push into many
/// peers' queues in turn; an await on one full queue would stall the others.
pub(crate) struct SignalQueue {
    sender: mpsc::Sender<RtcServerSignal>,
    overloaded: AtomicBool,
}

impl SignalQueue {
    pub(crate) fn new(sender: mpsc::Sender<RtcServerSignal>) -> Self {
        Self {
            sender,
            overloaded: AtomicBool::new(false),
        }
    }

    pub(crate) fn push(&self, signal: RtcServerSignal) -> SignalPush {
        match self.sender.try_send(signal) {
            Ok(()) => SignalPush::Queued,
            Err(TrySendError::Full(_)) => SignalPush::Overloaded {
                first: !self.overloaded.swap(true, Ordering::AcqRel),
            },
            Err(TrySendError::Closed(_)) => SignalPush::Closed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offer() -> RtcServerSignal {
        RtcServerSignal::Offer {
            sdp: "v=0".to_owned(),
        }
    }

    #[tokio::test]
    async fn full_queues_report_overload_once_without_waiting() {
        let (sender, mut receiver) = mpsc::channel(2);
        let queue = SignalQueue::new(sender);
        assert_eq!(queue.push(offer()), SignalPush::Queued);
        assert_eq!(queue.push(offer()), SignalPush::Queued);
        assert_eq!(queue.push(offer()), SignalPush::Overloaded { first: true });
        assert_eq!(queue.push(offer()), SignalPush::Overloaded { first: false });
        assert!(receiver.recv().await.is_some());
        assert_eq!(queue.push(offer()), SignalPush::Queued);
        drop(receiver);
        assert_eq!(queue.push(offer()), SignalPush::Closed);
    }
}
