use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

use crate::features::live_chat::{
    domain::actor::ChatActor,
    service::{cache::LiveChatServerEvent, live_chat_service::LiveChatService},
};

pub(super) async fn cleanup_live_chat_connection(
    service: Arc<LiveChatService>,
    connection_id: Uuid,
    actor: &ChatActor,
) {
    service.cache.unregister_connection(connection_id).await;
    service.clear_actor_typing(&actor.actor_key).await;
    service.cache.broadcast(LiveChatServerEvent::Presence {
        connected_count: service.cache.connected_count(),
    });
    info!(connection_id = %connection_id, "Live chat WebSocket disconnected");
}
