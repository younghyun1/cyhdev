use std::sync::Arc;

use chrono::{Duration as ChronoDuration, Utc};
use tracing::info;
use uuid::Uuid;

use crate::features::live_chat::{
    domain::{actor::ChatActor, message::DEFAULT_LIVE_CHAT_ROOM},
    service::{
        cache::{LiveChatServerEvent, TypingState},
        live_chat_service::LiveChatService,
    },
};

use super::LIVE_CHAT_TYPING_TTL_SECONDS;

pub(super) async fn handle_typing(service: Arc<LiveChatService>, actor: ChatActor, is_typing: bool) {
    let expires_at = Utc::now() + ChronoDuration::seconds(LIVE_CHAT_TYPING_TTL_SECONDS);
    let changed = if is_typing {
        service.cache
            .set_typing(TypingState {
                actor: actor.clone(),
                room_key: DEFAULT_LIVE_CHAT_ROOM.to_string(),
                expires_at,
            })
            .await
    } else {
        service.cache.clear_typing(&actor.actor_key).await
    };

    if is_typing || changed {
        broadcast_typing_set(service, expires_at).await;
    }
}

pub(super) async fn broadcast_typing_set(
    service: Arc<LiveChatService>,
    expires_at: chrono::DateTime<Utc>,
) {
    let actors = service.cache.active_typing_actors(Utc::now()).await;
    service.cache.broadcast(LiveChatServerEvent::TypingSet { actors, expires_at });
}

pub(super) async fn cleanup_live_chat_connection(
    service: Arc<LiveChatService>,
    connection_id: Uuid,
    actor: &ChatActor,
) {
    service.cache.unregister_connection(connection_id).await;
    let typing_changed = service.cache.clear_typing(&actor.actor_key).await;
    if typing_changed {
        let expires_at = Utc::now() + ChronoDuration::seconds(LIVE_CHAT_TYPING_TTL_SECONDS);
        broadcast_typing_set(Arc::clone(&service), expires_at).await;
    }
    service.cache.broadcast(LiveChatServerEvent::Presence {
            connected_count: service.cache.connected_count(),
        });
    info!(connection_id = %connection_id, "Live chat WebSocket disconnected");
}
