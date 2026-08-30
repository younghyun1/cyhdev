use std::{
    net::IpAddr,
    sync::atomic::Ordering,
};

use chrono::Utc;
use uuid::Uuid;

use super::{CachedLiveChatBan, LiveChatCache};
use crate::domain::live_chat::ban::LiveChatBan;

pub const LIVE_CHAT_BAN_INDEX_MAX_ENTRIES: usize = 50_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BanCacheLookup {
    Banned,
    NotBanned,
    DatabaseReadThroughRequired,
}

#[derive(Clone, Copy)]
pub struct LiveChatBanCacheStats {
    pub user_entries: usize,
    pub ip_entries: usize,
    pub max_entries_per_index: usize,
    pub complete: bool,
    pub hits: u64,
    pub misses: u64,
    pub database_read_throughs: u64,
    pub rejected_admissions: u64,
}

fn replacement_is_stronger(current: &CachedLiveChatBan, candidate: &CachedLiveChatBan) -> bool {
    match (current.expires_at, candidate.expires_at) {
        (None, _) => false,
        (Some(_), None) => true,
        (Some(current_expiry), Some(candidate_expiry)) => {
            candidate_expiry > current_expiry
                || (candidate_expiry == current_expiry && candidate.banned_at > current.banned_at)
        }
    }
}

impl LiveChatCache {
    pub async fn sync_bans(&self, bans: Vec<LiveChatBan>, source_complete: bool) {
        let mutation = self.ban_mutation.lock().await;
        self.bans_by_user.clear_async().await;
        self.bans_by_ip.clear_async().await;
        self.ban_cache_complete
            .store(source_complete, Ordering::Release);
        let now = Utc::now();

        for ban in bans {
            let cached_ban = CachedLiveChatBan::from(ban);
            if cached_ban.is_active(now) && !self.cache_ban_locked(&cached_ban).await {
                self.ban_cache_complete.store(false, Ordering::Release);
            }
        }
        drop(mutation);
    }

    pub async fn cache_ban(&self, ban: CachedLiveChatBan) -> bool {
        let mutation = self.ban_mutation.lock().await;
        let admitted = self.cache_ban_locked(&ban).await;
        if !admitted {
            self.ban_cache_complete.store(false, Ordering::Release);
        }
        drop(mutation);
        admitted
    }

    async fn cache_ban_locked(&self, ban: &CachedLiveChatBan) -> bool {
        let mut fully_admitted = true;
        if let Some(user_id) = ban.user_id {
            let updated = self
                .bans_by_user
                .update_async(&user_id, |_, current| {
                    if replacement_is_stronger(current, ban) {
                        *current = ban.clone();
                    }
                })
                .await
                .is_some();
            if !updated && self.bans_by_user.len() < LIVE_CHAT_BAN_INDEX_MAX_ENTRIES {
                let _ = self.bans_by_user.insert_async(user_id, ban.clone()).await;
            } else if !updated {
                fully_admitted = false;
                self.ban_rejected_admissions.fetch_add(1, Ordering::Relaxed);
            }
        }

        if let Some(ip) = ban.banned_ip {
            let updated = self
                .bans_by_ip
                .update_async(&ip, |_, current| {
                    if replacement_is_stronger(current, ban) {
                        *current = ban.clone();
                    }
                })
                .await
                .is_some();
            if !updated && self.bans_by_ip.len() < LIVE_CHAT_BAN_INDEX_MAX_ENTRIES {
                let _ = self.bans_by_ip.insert_async(ip, ban.clone()).await;
            } else if !updated {
                fully_admitted = false;
                self.ban_rejected_admissions.fetch_add(1, Ordering::Relaxed);
            }
        }
        fully_admitted
    }

    pub async fn lookup_ban(&self, user_id: Option<Uuid>, ip: IpAddr) -> BanCacheLookup {
        let now = Utc::now();
        if let Some(user_id) = user_id {
            match self
                .bans_by_user
                .read_async(&user_id, |_, ban| ban.is_active(now))
                .await
            {
                Some(true) => {
                    self.ban_cache_hits.fetch_add(1, Ordering::Relaxed);
                    return BanCacheLookup::Banned;
                }
                Some(false) => {
                    let _ = self
                        .bans_by_user
                        .remove_if_async(&user_id, |ban| !ban.is_active(now))
                        .await;
                }
                None => {}
            }
        }

        match self.bans_by_ip.read_async(&ip, |_, ban| ban.is_active(now)).await {
            Some(true) => {
                self.ban_cache_hits.fetch_add(1, Ordering::Relaxed);
                return BanCacheLookup::Banned;
            }
            Some(false) => {
                let _ = self
                    .bans_by_ip
                    .remove_if_async(&ip, |ban| !ban.is_active(now))
                    .await;
            }
            None => {}
        }

        self.ban_cache_misses.fetch_add(1, Ordering::Relaxed);
        if self.ban_cache_complete.load(Ordering::Acquire) {
            BanCacheLookup::NotBanned
        } else {
            BanCacheLookup::DatabaseReadThroughRequired
        }
    }

    pub fn record_ban_database_read_through(&self) {
        self.ban_database_read_throughs
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn ban_stats(&self) -> LiveChatBanCacheStats {
        LiveChatBanCacheStats {
            user_entries: self.bans_by_user.len(),
            ip_entries: self.bans_by_ip.len(),
            max_entries_per_index: LIVE_CHAT_BAN_INDEX_MAX_ENTRIES,
            complete: self.ban_cache_complete.load(Ordering::Acquire),
            hits: self.ban_cache_hits.load(Ordering::Relaxed),
            misses: self.ban_cache_misses.load(Ordering::Relaxed),
            database_read_throughs: self.ban_database_read_throughs.load(Ordering::Relaxed),
            rejected_admissions: self.ban_rejected_admissions.load(Ordering::Relaxed),
        }
    }
}
