use std::net::IpAddr;

use tracing::{error, info};
use uuid::Uuid;

use super::{
    cache::{
        BanCacheLookup, CachedChatMessage, CachedLiveChatBan, ChatActor,
        LIVE_CHAT_BAN_INDEX_MAX_ENTRIES,
    },
    live_chat_service::LiveChatService,
    super::error::LiveChatError,
};

impl LiveChatService {
    pub async fn prune_runtime(&self, now: chrono::DateTime<chrono::Utc>) {
        self.cache.clear_expired_rate_windows(now).await;
        self.cache.clear_expired_typing(now).await;
    }

    pub async fn enrich_messages(&self, messages: &mut [CachedChatMessage]) -> Result<(), LiveChatError> {
        let user_ids = messages.iter().filter_map(|message| message.user_id).collect::<Vec<_>>();
        let presentation = self.repository.user_presentation(&user_ids).await?;
        let country_codes = presentation.country_codes.values().copied().collect::<Vec<_>>();
        let flags = self.country_flags.country_flags(&country_codes).await;
        for message in messages {
            if let Some(user_id) = message.user_id
                && presentation.deleted_user_ids.contains(&user_id)
            {
                let _ = message.anonymize_deleted_user(user_id);
                continue;
            }
            if message.sender_country_flag.is_none() {
                message.sender_country_flag = match message.user_id {
                    Some(user_id) => match presentation.country_codes.get(&user_id) {
                        Some(code) => flags.get(code).cloned(),
                        None => None,
                    },
                    None => match message.guest_ip.and_then(|ip| self.geo_ip.country_alpha2(ip)) {
                        Some(code) => self.alpha2_flags.flag(&code).await,
                        None => None,
                    },
                };
            }
            if message.user_profile_picture_url.is_none()
                && let Some(user_id) = message.user_id
                && let Some(url) = presentation.profile_urls.get(&user_id)
            {
                message.user_profile_picture_url = Some(url.clone());
            }
        }
        Ok(())
    }

    pub async fn page_messages(
        &self,
        before: Option<Uuid>,
        limit: usize,
    ) -> Result<Vec<CachedChatMessage>, LiveChatError> {
        let limit = limit.clamp(1, 100);
        let mut messages = match before {
            Some(before) => self.repository.messages_before(before, limit).await?
                .into_iter().map(CachedChatMessage::from).collect(),
            None => self.cache.get_recent_chat_messages(limit).await,
        };
        self.enrich_messages(&mut messages).await?;
        Ok(messages)
    }

    pub async fn synchronize_messages(&self) -> Result<usize, LiveChatError> {
        let rows = self.repository.recent_messages(50_000).await?;
        self.cache.clear_messages().await;
        let count = rows.len();
        let mut messages = rows.into_iter().map(CachedChatMessage::from).collect::<Vec<_>>();
        self.enrich_messages(&mut messages).await?;
        for message in messages { self.cache.append_persisted_chat_message(message).await; }
        info!(rows_synchronized = count, "Synchronized live chat cache");
        Ok(count)
    }

    pub async fn synchronize_bans(&self) -> Result<usize, LiveChatError> {
        let mut bans = self.repository.active_bans((LIVE_CHAT_BAN_INDEX_MAX_ENTRIES + 1) as i64).await?;
        let source_complete = bans.len() <= LIVE_CHAT_BAN_INDEX_MAX_ENTRIES;
        if !source_complete { bans.truncate(LIVE_CHAT_BAN_INDEX_MAX_ENTRIES); }
        let count = bans.len(); self.cache.sync_bans(bans, source_complete).await;
        Ok(count)
    }

    pub async fn is_actor_banned(&self, user_id: Option<Uuid>, ip: IpAddr) -> bool {
        match self.cache.lookup_ban(user_id, ip).await {
            BanCacheLookup::Banned => return true,
            BanCacheLookup::NotBanned => return false,
            BanCacheLookup::DatabaseReadThroughRequired => {}
        }
        self.cache.record_ban_database_read_through();
        match self.repository.active_ban_for(user_id, ip).await {
            Ok(Some(ban)) => {
                let _ = self.cache.cache_ban(CachedLiveChatBan::from(ban)).await; true
            }
            Ok(None) => false,
            Err(error_value) => {
                error!(error = %error_value, user_id = ?user_id, client_ip = %ip, "Live chat ban read-through failed closed"); true
            }
        }
    }

    pub async fn persist_message(&self, actor: &ChatActor, body: String) -> Option<CachedChatMessage> {
        match self.repository.insert_message(actor, body).await {
            Ok(message) => {
                let mut cached = CachedChatMessage::from(message);
                cached.sender_country_flag = actor.country_flag.clone();
                cached.user_profile_picture_url = actor.user_profile_picture_url.clone();
                if let Some(user_id) = cached.user_id
                    && self.cache.is_connected_user_disabled(user_id)
                { let _ = cached.anonymize_deleted_user(user_id); }
                Some(cached)
            }
            Err(error_value) => { error!(error = %error_value, user_id = ?actor.user_id, "Failed to persist live chat message"); None }
        }
    }

    pub async fn persist_abuse_ban(&self, actor: &ChatActor, ip: IpAddr) -> Option<CachedLiveChatBan> {
        match self.repository.insert_abuse_ban(actor, ip).await {
            Ok(ban) => Some(CachedLiveChatBan::from(ban)),
            Err(error_value) => { error!(error = %error_value, user_id = ?actor.user_id, client_ip = %ip, "Failed to persist live chat ban"); None }
        }
    }
}
