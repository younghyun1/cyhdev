//! Live-chat ban persistence.

use std::net::IpAddr;

use chrono::{DateTime, Utc};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper,
};
use diesel_async::{AsyncConnection, RunQueryDsl};
use ipnet::IpNet;
use uuid::Uuid;

use crate::schema::live_chat_bans;

use super::{
    super::{
        domain::{
            actor::ChatActor,
            ban::{LIVE_CHAT_BAN_SOURCE_ABNORMAL_MESSAGING, LiveChatBan},
            ip_prefix::ban_networks_for,
        },
        error::LiveChatError,
    },
    live_chat_repository::LiveChatRepository,
    messages::lock_active_user,
    records::{BanRecord, NewBanRecord},
};

impl LiveChatRepository {
    /// Record an automatic abnormal-messaging ban covering `banned_network`
    /// (the sender's /32 or IPv6 /64) and, when signed in, their account.
    pub async fn insert_abuse_ban(
        &self,
        actor: &ChatActor,
        banned_network: IpNet,
        expires_at: DateTime<Utc>,
    ) -> Result<LiveChatBan, LiveChatError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<LiveChatBan, LiveChatError, _>(async move |connection| {
                if let Some(user_id) = actor.user_id {
                    lock_active_user(connection, user_id).await?;
                }
                diesel::insert_into(live_chat_bans::table)
                    .values(NewBanRecord {
                        live_chat_ban_id: Uuid::now_v7(),
                        user_id: actor.user_id,
                        banned_ip: Some(banned_network),
                        reason: "More than 10 live chat message events in one second.".to_owned(),
                        ban_source: LIVE_CHAT_BAN_SOURCE_ABNORMAL_MESSAGING.to_owned(),
                        banned_at: Utc::now(),
                        expires_at: Some(expires_at),
                    })
                    .returning(BanRecord::as_returning())
                    .get_result::<BanRecord>(&mut *connection)
                    .await
                    .map(LiveChatBan::from)
                    .map_err(LiveChatError::Database)
            })
            .await
    }

    pub async fn active_bans(&self, limit: i64) -> Result<Vec<LiveChatBan>, LiveChatError> {
        let mut connection = self.connection().await?;
        let now = Utc::now();
        live_chat_bans::table
            .filter(
                live_chat_bans::expires_at
                    .is_null()
                    .or(live_chat_bans::expires_at.gt(now)),
            )
            .order((
                live_chat_bans::banned_at.desc(),
                live_chat_bans::live_chat_ban_id.desc(),
            ))
            .limit(limit)
            .select(BanRecord::as_select())
            .load::<BanRecord>(&mut connection)
            .await
            .map(|rows| rows.into_iter().map(LiveChatBan::from).collect())
            .map_err(LiveChatError::Database)
    }

    /// Read-through for an incomplete ban cache. Matches the same networks the
    /// cache consults (group network plus legacy host entry) with equality, so
    /// the partial `banned_ip` B-tree index still serves the lookup.
    pub async fn active_ban_for(
        &self,
        user_id: Option<Uuid>,
        ip: IpAddr,
    ) -> Result<Option<LiveChatBan>, LiveChatError> {
        let mut connection = self.connection().await?;
        let now = Utc::now();
        let networks = ban_networks_for(ip).iter().collect::<Vec<_>>();
        let mut query = live_chat_bans::table
            .filter(
                live_chat_bans::expires_at
                    .is_null()
                    .or(live_chat_bans::expires_at.gt(now)),
            )
            .into_boxed();
        query = match user_id {
            Some(id) => query.filter(
                live_chat_bans::user_id
                    .eq(Some(id))
                    .or(live_chat_bans::banned_ip.eq_any(networks)),
            ),
            None => query.filter(live_chat_bans::banned_ip.eq_any(networks)),
        };
        query
            .order(live_chat_bans::banned_at.desc())
            .select(BanRecord::as_select())
            .first::<BanRecord>(&mut connection)
            .await
            .optional()
            .map(|row| row.map(LiveChatBan::from))
            .map_err(LiveChatError::Database)
    }
}
