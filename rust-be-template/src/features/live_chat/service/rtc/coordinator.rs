use std::sync::Arc;

use tracing::error;
use uuid::Uuid;

use super::super::super::{
    domain::actor::ChatActor, repository::live_chat_repository::LiveChatRepository,
    service::cache::LiveChatCache,
};
use super::{
    engine::RtcEngine,
    room::{RtcRoom, RtcRoomAcquire},
};

pub struct RtcCoordinator {
    engine: Option<Arc<RtcEngine>>,
    max_participants: usize,
    rooms: scc::HashMap<String, Arc<RtcRoom>>,
    cache: Arc<LiveChatCache>,
    repository: Arc<LiveChatRepository>,
}

impl RtcCoordinator {
    pub fn new(
        engine: Option<Arc<RtcEngine>>,
        max_participants: usize,
        cache: Arc<LiveChatCache>,
        repository: Arc<LiveChatRepository>,
    ) -> Self {
        Self {
            engine,
            max_participants,
            rooms: scc::HashMap::new(),
            cache,
            repository,
        }
    }

    pub fn enabled(&self) -> bool {
        self.engine.is_some()
    }
    pub fn engine(&self) -> Option<Arc<RtcEngine>> {
        self.engine.clone()
    }
    pub fn max_participants(&self) -> usize {
        self.max_participants
    }

    pub async fn acquire_room(&self, room_key: &str, user_id: Option<Uuid>) -> RtcRoomAcquire {
        if self.engine.is_none() {
            return RtcRoomAcquire::Unavailable;
        }
        for _ in 0..8 {
            if let Some(room) = self
                .rooms
                .read_async(room_key, |_, room| room.clone())
                .await
            {
                if !room.try_reserve() {
                    return RtcRoomAcquire::Full;
                }
                if room.is_removed() {
                    room.release_slot();
                    continue;
                }
                return RtcRoomAcquire::Acquired(room);
            }
            let call_id = match self.repository.open_call(room_key, user_id).await {
                Ok(call_id) => call_id,
                Err(error_value) => {
                    error!(error = %error_value, room_key, "Failed to open live chat call row");
                    return RtcRoomAcquire::Unavailable;
                }
            };
            let room = RtcRoom::new(
                room_key.to_owned(),
                call_id,
                self.cache.broadcast_sender(),
                self.max_participants,
            );
            let _ = room.try_reserve();
            match self
                .rooms
                .insert_async(room_key.to_owned(), room.clone())
                .await
            {
                Ok(_) => return RtcRoomAcquire::Acquired(room),
                Err(_) => {
                    room.release_slot();
                    let _ = self.repository.close_call(call_id).await;
                    continue;
                }
            }
        }
        RtcRoomAcquire::Unavailable
    }

    pub async fn remove_room_if_empty(&self, room_key: &str) {
        let empty = self
            .rooms
            .read_async(room_key, |_, room| room.occupancy() == 0)
            .await
            .unwrap_or(false);
        if !empty {
            return;
        }
        if let Some((_, room)) = self.rooms.remove_async(room_key).await {
            if room.occupancy() == 0 {
                room.mark_removed();
                let _ = self.repository.close_call(room.call_id).await;
            } else {
                let _ = self.rooms.insert_async(room_key.to_owned(), room).await;
            }
        }
    }

    pub async fn prune_empty_rooms(&self) {
        let mut keys = Vec::new();
        self.rooms
            .iter_async(|key, room| {
                if room.occupancy() == 0 {
                    keys.push(key.clone());
                }
                true
            })
            .await;
        for key in keys {
            self.remove_room_if_empty(&key).await;
        }
    }

    pub async fn participant_join(
        &self,
        call_id: Uuid,
        actor: &ChatActor,
        audio: bool,
        video: bool,
    ) -> Option<Uuid> {
        self.repository.join_call(call_id, actor, audio, video).await
            .map_err(|error_value| error!(error = %error_value, user_id = ?actor.user_id, "Failed to persist call participant join")).ok()
    }

    pub async fn participant_leave(&self, participant_id: Uuid) {
        if let Err(error_value) = self.repository.leave_call(participant_id).await {
            error!(error = %error_value, %participant_id, "Failed to persist call participant leave");
        }
    }
}
