//! Room broadcasts shared by every connection writer.

use std::sync::{Arc, OnceLock};

use super::LiveChatServerEvent;

/// One room broadcast whose wire frames are encoded at most once per protocol.
///
/// The first writer that needs a protocol's frame encodes it and every other
/// recipient shares the same bytes, so fan-out cost no longer scales encoding
/// work with the number of connections. Deleted-account anonymization runs at
/// that first encode; a frame encoded before a deletion keeps its identity,
/// as a frame already on the wire would, and lives only as long as the
/// bounded broadcast ring holds it.
pub struct LiveChatBroadcast {
    event: LiveChatServerEvent,
    binary_frame: OnceLock<Option<Arc<[u8]>>>,
    json_frame: OnceLock<Option<Arc<[u8]>>>,
}

impl LiveChatBroadcast {
    pub fn new(event: LiveChatServerEvent) -> Arc<Self> {
        Arc::new(Self {
            event,
            binary_frame: OnceLock::new(),
            json_frame: OnceLock::new(),
        })
    }

    pub fn event(&self) -> &LiveChatServerEvent {
        &self.event
    }

    /// Shared binary frame; `encode` runs only for the first caller and a
    /// `None` result (encoding failure) is cached as well.
    pub fn binary_frame(
        &self,
        encode: impl FnOnce(&LiveChatServerEvent) -> Option<Vec<u8>>,
    ) -> Option<Arc<[u8]>> {
        self.binary_frame
            .get_or_init(|| encode(&self.event).map(Arc::from))
            .clone()
    }

    /// Shared UTF-8 JSON frame with the same encode-once semantics.
    pub fn json_frame(
        &self,
        encode: impl FnOnce(&LiveChatServerEvent) -> Option<Vec<u8>>,
    ) -> Option<Arc<[u8]>> {
        self.json_frame
            .get_or_init(|| encode(&self.event).map(Arc::from))
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test]
    fn each_protocol_encodes_once_and_shares_bytes() {
        let broadcast =
            LiveChatBroadcast::new(LiveChatServerEvent::Presence { connected_count: 3 });
        let encodes = AtomicUsize::new(0);
        let encode = |_: &LiveChatServerEvent| {
            encodes.fetch_add(1, Ordering::SeqCst);
            Some(vec![1, 2, 3])
        };
        let first = broadcast.binary_frame(encode);
        let second = broadcast.binary_frame(encode);
        assert_eq!(encodes.load(Ordering::SeqCst), 1);
        match (first, second) {
            (Some(first), Some(second)) => assert!(Arc::ptr_eq(&first, &second)),
            _ => panic!("binary frame should encode"),
        }
        let _ = broadcast.json_frame(encode);
        let _ = broadcast.json_frame(encode);
        assert_eq!(encodes.load(Ordering::SeqCst), 2);
    }
}
