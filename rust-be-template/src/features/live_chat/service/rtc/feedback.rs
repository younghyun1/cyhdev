//! Deliver only keyframe feedback to the SFU after default interceptors process RTCP.

use std::collections::VecDeque;
use std::time::Instant;

use rtc::interceptor::{Attribute, Interceptor, Packet, StreamInfo, TaggedPacket};
use rtc::rtcp::payload_feedbacks::full_intra_request::FullIntraRequest;
use rtc::rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication;
use rtc::sansio::Protocol;
use rtc::shared::error::Error;

const MAX_PENDING_PACKETS: usize = 64;

/// RTC 0.21 consumes inbound RTCP unless an interceptor marks application feedback.
#[derive(Default)]
pub(super) struct KeyframeFeedback {
    reads: VecDeque<TaggedPacket>,
    writes: VecDeque<TaggedPacket>,
}

impl Protocol<TaggedPacket, TaggedPacket, ()> for KeyframeFeedback {
    type Rout = TaggedPacket;
    type Wout = TaggedPacket;
    type Eout = ();
    type Error = Error;
    type Time = Instant;

    fn handle_read(&mut self, mut packet: TaggedPacket) -> Result<(), Error> {
        if let Packet::Rtcp(feedback) = &mut packet.message.packet {
            feedback.retain(|item| {
                item.as_any().is::<PictureLossIndication>()
                    || item.as_any().is::<FullIntraRequest>()
            });
            if feedback.is_empty() {
                return Ok(());
            }
            packet.message.add(Attribute::DeliverToApplication);
        }
        enqueue(&mut self.reads, packet)
    }

    fn poll_read(&mut self) -> Option<TaggedPacket> {
        self.reads.pop_front()
    }

    fn handle_write(&mut self, packet: TaggedPacket) -> Result<(), Error> {
        enqueue(&mut self.writes, packet)
    }

    fn poll_write(&mut self) -> Option<TaggedPacket> {
        self.writes.pop_front()
    }
}

impl Interceptor for KeyframeFeedback {
    fn bind_local_stream(&mut self, _: &StreamInfo) {}
    fn unbind_local_stream(&mut self, _: &StreamInfo) {}
    fn bind_remote_stream(&mut self, _: &StreamInfo) {}
    fn unbind_remote_stream(&mut self, _: &StreamInfo) {}
}

fn enqueue(queue: &mut VecDeque<TaggedPacket>, packet: TaggedPacket) -> Result<(), Error> {
    if queue.len() >= MAX_PENDING_PACKETS {
        return Err(Error::ErrBufferFull);
    }
    queue.push_back(packet);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rtc::interceptor::AttributedPacket;
    use rtc::rtcp::receiver_report::ReceiverReport;

    fn tagged(packet: Packet) -> TaggedPacket {
        TaggedPacket {
            now: Instant::now(),
            transport: Default::default(),
            message: AttributedPacket::new(packet),
        }
    }

    #[test]
    fn forwards_only_keyframe_requests() -> Result<(), Error> {
        let mut interceptor = KeyframeFeedback::default();
        interceptor.handle_read(tagged(Packet::Rtcp(vec![
            Box::new(ReceiverReport::default()),
            Box::new(PictureLossIndication::default()),
            Box::new(FullIntraRequest::default()),
        ])))?;
        let forwarded = interceptor.poll_read().ok_or(Error::ErrBufferClosed)?;
        assert!(forwarded.message.has(&Attribute::DeliverToApplication));
        assert!(
            matches!(forwarded.message.packet, Packet::Rtcp(ref packets) if packets.len() == 2)
        );
        interceptor.handle_read(tagged(Packet::Rtcp(vec![Box::new(
            ReceiverReport::default(),
        )])))?;
        assert!(interceptor.poll_read().is_none());
        Ok(())
    }

    #[test]
    fn preserves_media_and_bounds_queues() -> Result<(), Error> {
        let mut interceptor = KeyframeFeedback::default();
        for _ in 0..MAX_PENDING_PACKETS {
            interceptor.handle_write(tagged(Packet::Rtp(Default::default())))?;
            interceptor.handle_read(tagged(Packet::Rtp(Default::default())))?;
        }
        assert!(matches!(
            interceptor.handle_write(tagged(Packet::Rtp(Default::default()))),
            Err(Error::ErrBufferFull)
        ));
        assert!(matches!(
            interceptor.handle_read(tagged(Packet::Rtp(Default::default()))),
            Err(Error::ErrBufferFull)
        ));
        let received = interceptor.poll_read().ok_or(Error::ErrBufferClosed)?;
        assert!(matches!(received.message.packet, Packet::Rtp(_)));
        assert!(!received.message.has(&Attribute::DeliverToApplication));
        assert!(interceptor.poll_write().is_some());
        Ok(())
    }
}
