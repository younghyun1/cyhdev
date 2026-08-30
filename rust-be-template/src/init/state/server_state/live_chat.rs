use std::collections::{HashMap as StdHashMap, HashSet as StdHashSet};

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    OptionalExtension, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use tracing::{error, info};
use uuid::Uuid;

use super::ServerState;
use crate::domain::live_chat::{
    ban::LiveChatBan,
    cache::{
        BanCacheLookup, CachedChatMessage, CachedLiveChatBan,
        LIVE_CHAT_BAN_INDEX_MAX_ENTRIES,
    },
    message::LiveChatMessage,
};
use crate::schema::{live_chat_bans, live_chat_messages, user_profile_pictures, users};
use crate::util::time::now::tokio_now;

type LiveChatUserPresentationRow = (
    Uuid,
    i32,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<String>,
);

impl ServerState {
    pub async fn enrich_live_chat_message_flags(
        &self,
        messages: &mut [CachedChatMessage],
    ) -> anyhow::Result<()> {
        let mut user_ids = Vec::new();
        let mut seen_user_ids: StdHashMap<Uuid, ()> = StdHashMap::new();
        for message in messages.iter() {
            if let Some(user_id) = message.user_id
                && !seen_user_ids.contains_key(&user_id)
            {
                seen_user_ids.insert(user_id, ());
                user_ids.push(user_id);
            }
        }

        let mut user_country_codes: StdHashMap<Uuid, i32> = StdHashMap::new();
        let mut user_profile_picture_urls: StdHashMap<Uuid, String> = StdHashMap::new();
        let mut deleted_user_ids: StdHashSet<Uuid> = StdHashSet::new();
        if !user_ids.is_empty() {
            // Missing tombstones fail closed to the deleted presentation.
            deleted_user_ids.extend(user_ids.iter().copied());
            let mut conn = self.get_conn().await?;
            let rows: Vec<LiveChatUserPresentationRow> = users::table
                .left_join(user_profile_pictures::table.on(
                    user_profile_pictures::user_id
                        .eq(users::user_id)
                        .and(user_profile_pictures::user_profile_picture_is_active.eq(true)),
                ))
                .filter(users::user_id.eq_any(&user_ids))
                .select((
                    users::user_id,
                    users::user_country,
                    users::user_deleted_at,
                    user_profile_pictures::user_profile_picture_link.nullable(),
                ))
                .load(&mut conn)
                .await?;

            for (user_id, user_country, deleted_at, profile_picture_url) in rows {
                if deleted_at.is_some() {
                    deleted_user_ids.insert(user_id);
                } else {
                    deleted_user_ids.remove(&user_id);
                    user_country_codes.insert(user_id, user_country);
                    if let Some(profile_picture_url) = profile_picture_url {
                        user_profile_picture_urls.insert(user_id, profile_picture_url);
                    }
                }
            }
            drop(conn);
        }

        let country_map = self.country_map.read().await;
        for message in messages.iter_mut() {
            if let Some(user_id) = message.user_id
                && deleted_user_ids.contains(&user_id)
            {
                let _ = message.anonymize_deleted_user(user_id);
                continue;
            }
            let country_flag = match (message.sender_country_flag.clone(), message.user_id) {
                (Some(country_flag), _) => Some(country_flag),
                (None, Some(user_id)) => user_country_codes
                    .get(&user_id)
                    .and_then(|country_code| country_map.get_flag_by_code(*country_code)),
                (None, None) => match message.guest_ip {
                    Some(guest_ip) => self.lookup_ip_location(guest_ip).and_then(|ip_info| {
                        country_map
                            .lookup_by_alpha2(&ip_info.country_code)
                            .map(|country| country.country.country_flag.clone())
                    }),
                    None => None,
                },
            };
            message.sender_country_flag = country_flag;

            if message.user_profile_picture_url.is_none()
                && let Some(user_id) = message.user_id
                && let Some(user_profile_picture_url) = user_profile_picture_urls.get(&user_id)
            {
                message.user_profile_picture_url = Some(user_profile_picture_url.clone());
            }
        }

        Ok(())
    }

    pub async fn sync_live_chat_ban_cache(&self) -> anyhow::Result<usize> {
        let start = tokio_now();
        let now = chrono::Utc::now();
        let mut conn = self.get_conn().await?;

        let mut rows: Vec<LiveChatBan> = live_chat_bans::table
            .filter(
                live_chat_bans::expires_at
                    .is_null()
                    .or(live_chat_bans::expires_at.gt(now)),
            )
            .order((
                live_chat_bans::banned_at.desc(),
                live_chat_bans::live_chat_ban_id.desc(),
            ))
            .limit((LIVE_CHAT_BAN_INDEX_MAX_ENTRIES + 1) as i64)
            .select(LiveChatBan::as_select())
            .load(&mut conn)
            .await?;

        drop(conn);

        let source_complete = rows.len() <= LIVE_CHAT_BAN_INDEX_MAX_ENTRIES;
        if !source_complete {
            rows.truncate(LIVE_CHAT_BAN_INDEX_MAX_ENTRIES);
        }
        let row_count = rows.len();
        self.live_chat_cache.sync_bans(rows, source_complete).await;
        let stats = self.live_chat_cache.ban_stats();

        info!(
            elapsed = ?start.elapsed(),
            rows_synchronized = row_count,
            user_entries = stats.user_entries,
            ip_entries = stats.ip_entries,
            max_entries_per_index = stats.max_entries_per_index,
            cache_complete = stats.complete,
            cache_hits = stats.hits,
            cache_misses = stats.misses,
            database_read_throughs = stats.database_read_throughs,
            rejected_admissions = stats.rejected_admissions,
            "Synchronized live chat ban cache"
        );

        Ok(row_count)
    }

    pub async fn is_live_chat_actor_banned(&self, user_id: Option<Uuid>, ip: std::net::IpAddr) -> bool {
        match self.live_chat_cache.lookup_ban(user_id, ip).await {
            BanCacheLookup::Banned => return true,
            BanCacheLookup::NotBanned => return false,
            BanCacheLookup::DatabaseReadThroughRequired => {}
        }
        self.live_chat_cache.record_ban_database_read_through();

        let mut conn = match self.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!(error = ?e, user_id = ?user_id, client_ip = %ip, "Live chat ban read-through failed closed");
                return true;
            }
        };
        let now = chrono::Utc::now();
        let ip_network = ipnet::IpNet::from(ip);
        let mut query = live_chat_bans::table
            .filter(
                live_chat_bans::expires_at
                    .is_null()
                    .or(live_chat_bans::expires_at.gt(now)),
            )
            .into_boxed();
        query = match user_id {
            Some(user_id) => query.filter(
                live_chat_bans::user_id
                    .eq(Some(user_id))
                    .or(live_chat_bans::banned_ip.eq(Some(ip_network))),
            ),
            None => query.filter(live_chat_bans::banned_ip.eq(Some(ip_network))),
        };
        let row = query
            .order(live_chat_bans::banned_at.desc())
            .select(LiveChatBan::as_select())
            .first::<LiveChatBan>(&mut conn)
            .await
            .optional();
        drop(conn);

        match row {
            Ok(Some(ban)) => {
                let _ = self.live_chat_cache.cache_ban(CachedLiveChatBan::from(ban)).await;
                true
            }
            Ok(None) => false,
            Err(e) => {
                error!(error = ?e, user_id = ?user_id, client_ip = %ip, "Live chat ban read-through failed closed");
                true
            }
        }
    }

    pub async fn sync_live_chat_cache(&self) -> anyhow::Result<usize> {
        const LIVE_CHAT_STARTUP_ROW_LIMIT: i64 = 50_000;

        let start = tokio_now();
        let mut conn = self.get_conn().await?;

        let mut rows: Vec<LiveChatMessage> = live_chat_messages::table
            .select(LiveChatMessage::as_select())
            .order(live_chat_messages::message_created_at.desc())
            .limit(LIVE_CHAT_STARTUP_ROW_LIMIT)
            .load(&mut conn)
            .await?;

        drop(conn);

        self.live_chat_cache.clear_messages().await;
        rows.reverse();
        let row_count = rows.len();
        let mut cached_messages = rows
            .into_iter()
            .map(CachedChatMessage::from)
            .collect::<Vec<CachedChatMessage>>();
        self.enrich_live_chat_message_flags(&mut cached_messages)
            .await?;

        for row in cached_messages {
            self.live_chat_cache
                .append_persisted_chat_message(row)
                .await;
        }

        info!(
            elapsed = ?start.elapsed(),
            rows_synchronized = %row_count,
            "Synchronized live chat cache."
        );

        Ok(row_count)
    }
}
