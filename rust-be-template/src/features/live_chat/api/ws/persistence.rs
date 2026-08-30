use std::{net::IpAddr, sync::Arc};

use crate::features::live_chat::{
    domain::actor::ChatActor,
    service::{
        cache::{CachedChatMessage, CachedLiveChatBan},
        live_chat_service::LiveChatService,
    },
};

pub(super) async fn persist_message(
    service: Arc<LiveChatService>,
    actor: &ChatActor,
    body: String,
) -> Option<CachedChatMessage> {
    service.persist_message(actor, body).await
}

pub(super) async fn persist_live_chat_ban(
    service: Arc<LiveChatService>,
    actor: &ChatActor,
    client_ip: IpAddr,
) -> Option<CachedLiveChatBan> {
    service.persist_abuse_ban(actor, client_ip).await
}
