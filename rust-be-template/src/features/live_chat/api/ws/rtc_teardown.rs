use std::sync::Arc;

use tokio::sync::mpsc;

use crate::features::live_chat::{
    domain::rtc::RtcServerSignal,
    service::{
        cache::LiveChatServerEvent,
        rtc::{peer::RtcPeer, room::RtcRoom},
    },
};

use super::{protocol::{encode_event, send_event}, rtc::RtcSession};

impl RtcSession {
    pub(super) async fn leave(&self) {
        let (room, _peer, participant_id) = {
            let mut inner = self.inner.lock().await;
            (inner.room.take(), inner.peer.take(), inner.participant_id.take())
        };
        if let Some(room) = room.as_ref() {
            let deleted_user_id = self.actor.user_id.filter(|user_id| {
                self.service.cache.is_connected_user_disabled(*user_id)
            });
            match deleted_user_id {
                Some(user_id) => room.teardown_deleted_user_peer(self.connection_id, user_id).await,
                None => room.teardown_peer(self.connection_id).await,
            }
        }
        if let Some(participant_id) = participant_id {
            self.service.rtc.participant_leave(participant_id).await;
        }
        if let Some(room) = room {
            self.service.rtc.remove_room_if_empty(&room.room_key).await;
        }
    }

    pub(super) fn spawn_signal_relay(&self, mut receiver: mpsc::Receiver<RtcServerSignal>) {
        let out = self.out.clone();
        let service = Arc::clone(&self.service);
        let wire_protocol = self.wire_protocol;
        tokio::spawn(async move {
            while let Some(signal) = receiver.recv().await {
                let mut event = LiveChatServerEvent::Rtc(signal);
                service.cache.anonymize_event_for_public(&mut event);
                if let Some(message) = encode_event(&event, wire_protocol)
                    && out.send(message).await.is_err()
                {
                    break;
                }
            }
        });
    }

    pub(super) async fn current_peer(&self) -> Option<Arc<RtcPeer>> {
        self.inner.lock().await.peer.clone()
    }

    pub(super) async fn current_room_peer(&self) -> (Option<Arc<RtcRoom>>, Option<Arc<RtcPeer>>) {
        let inner = self.inner.lock().await;
        (inner.room.clone(), inner.peer.clone())
    }

    pub(super) async fn send_error(&self, code: &str, message: &str) {
        let event = LiveChatServerEvent::Rtc(RtcServerSignal::Error {
            code: code.to_owned(), message: message.to_owned(),
        });
        send_event(&self.out, &event, self.wire_protocol).await;
    }
}
