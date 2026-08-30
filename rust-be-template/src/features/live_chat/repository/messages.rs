use std::{collections::{HashMap, HashSet}, net::IpAddr};

use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    schema::{live_chat_bans, live_chat_messages, user_profile_pictures, users},
};

use super::{
    live_chat_repository::LiveChatRepository,
    records::{BanRecord, MessageRecord, NewBanRecord, NewMessageRecord},
    super::{domain::{actor::ChatActor, ban::LiveChatBan, message::{DEFAULT_LIVE_CHAT_ROOM, LiveChatMessage}}, error::LiveChatError},
};

pub struct UserPresentation {
    pub country_codes: HashMap<Uuid, i32>,
    pub profile_urls: HashMap<Uuid, String>,
    pub deleted_user_ids: HashSet<Uuid>,
}

impl LiveChatRepository {
    pub async fn recent_messages(&self, limit: i64) -> Result<Vec<LiveChatMessage>, LiveChatError> {
        let mut connection = self.connection().await?;
        let mut rows = live_chat_messages::table
            .select(MessageRecord::as_select())
            .order((live_chat_messages::message_created_at.desc(), live_chat_messages::live_chat_message_id.desc()))
            .limit(limit.clamp(1, 50_000))
            .load::<MessageRecord>(&mut connection)
            .await?
            .into_iter().map(LiveChatMessage::from).collect::<Vec<_>>();
        rows.reverse();
        Ok(rows)
    }

    pub async fn messages_before(
        &self,
        before_message_id: Uuid,
        limit: usize,
    ) -> Result<Vec<LiveChatMessage>, LiveChatError> {
        let mut connection = self.connection().await?;
        let before = live_chat_messages::table
            .find(before_message_id)
            .select((live_chat_messages::room_key, live_chat_messages::message_created_at))
            .first::<(String, chrono::DateTime<Utc>)>(&mut connection)
            .await.optional()?.ok_or(LiveChatError::InvalidCursor)?;
        let mut rows = live_chat_messages::table
            .filter(live_chat_messages::room_key.eq(before.0))
            .filter(live_chat_messages::message_deleted_at.is_null())
            .filter(live_chat_messages::message_created_at.lt(before.1).or(
                live_chat_messages::message_created_at.eq(before.1)
                    .and(live_chat_messages::live_chat_message_id.lt(before_message_id))))
            .select(MessageRecord::as_select())
            .order((live_chat_messages::message_created_at.desc(), live_chat_messages::live_chat_message_id.desc()))
            .limit(i64::try_from(limit.clamp(1, 100)).unwrap_or(100))
            .load::<MessageRecord>(&mut connection).await?
            .into_iter().map(LiveChatMessage::from).collect::<Vec<_>>();
        rows.reverse();
        Ok(rows)
    }

    pub async fn user_presentation(
        &self,
        user_ids: &[Uuid],
    ) -> Result<UserPresentation, LiveChatError> {
        let mut ids = user_ids.to_vec();
        ids.sort_unstable(); ids.dedup();
        if ids.is_empty() {
            return Ok(UserPresentation { country_codes: HashMap::new(), profile_urls: HashMap::new(), deleted_user_ids: HashSet::new() });
        }
        let mut deleted_user_ids = ids.iter().copied().collect::<HashSet<_>>();
        let mut connection = self.connection().await?;
        let rows = users::table.left_join(user_profile_pictures::table.on(
            user_profile_pictures::user_id.eq(users::user_id)
                .and(user_profile_pictures::user_profile_picture_is_active.eq(true))))
            .filter(users::user_id.eq_any(ids))
            .select((users::user_id, users::user_country, users::user_deleted_at,
                user_profile_pictures::user_profile_picture_link.nullable()))
            .load::<(Uuid, i32, Option<chrono::DateTime<Utc>>, Option<String>)>(&mut connection).await?;
        let mut country_codes = HashMap::new();
        let mut profile_urls = HashMap::new();
        for (id, country, deleted, profile) in rows {
            if deleted.is_none() {
                deleted_user_ids.remove(&id); country_codes.insert(id, country);
                if let Some(profile) = profile { profile_urls.insert(id, profile); }
            }
        }
        Ok(UserPresentation { country_codes, profile_urls, deleted_user_ids })
    }

    pub async fn insert_message(&self, actor: &ChatActor, body: String) -> Result<LiveChatMessage, LiveChatError> {
        let mut connection = self.connection().await?;
        connection.transaction::<LiveChatMessage, LiveChatError, _>(async move |connection| {
            if let Some(user_id) = actor.user_id { lock_active_user(connection, user_id).await?; }
            diesel::insert_into(live_chat_messages::table).values(NewMessageRecord {
                live_chat_message_id: Uuid::now_v7(), room_key: DEFAULT_LIVE_CHAT_ROOM.to_owned(),
                user_id: actor.user_id, guest_ip: actor.guest_ip.map(ipnet::IpNet::from), sender_kind: actor.sender_kind,
                sender_display_name: actor.display_name.clone(), message_body: body, message_created_at: Utc::now(),
            }).returning(MessageRecord::as_returning()).get_result::<MessageRecord>(&mut *connection)
                .await.map(LiveChatMessage::from).map_err(LiveChatError::Database)
        }).await
    }

    pub async fn insert_abuse_ban(&self, actor: &ChatActor, ip: IpAddr) -> Result<LiveChatBan, LiveChatError> {
        let mut connection = self.connection().await?;
        connection.transaction::<LiveChatBan, LiveChatError, _>(async move |connection| {
            if let Some(user_id) = actor.user_id { lock_active_user(connection, user_id).await?; }
            diesel::insert_into(live_chat_bans::table).values(NewBanRecord {
                live_chat_ban_id: Uuid::now_v7(), user_id: actor.user_id, banned_ip: Some(ipnet::IpNet::from(ip)),
                reason: "More than 10 live chat message events in one second.".to_owned(),
                ban_source: super::super::domain::ban::LIVE_CHAT_BAN_SOURCE_ABNORMAL_MESSAGING.to_owned(),
                banned_at: Utc::now(), expires_at: None,
            }).returning(BanRecord::as_returning()).get_result::<BanRecord>(&mut *connection)
                .await.map(LiveChatBan::from).map_err(LiveChatError::Database)
        }).await
    }

    pub async fn active_bans(&self, limit: i64) -> Result<Vec<LiveChatBan>, LiveChatError> {
        let mut connection = self.connection().await?; let now = Utc::now();
        live_chat_bans::table.filter(live_chat_bans::expires_at.is_null().or(live_chat_bans::expires_at.gt(now)))
            .order((live_chat_bans::banned_at.desc(), live_chat_bans::live_chat_ban_id.desc()))
            .limit(limit).select(BanRecord::as_select()).load::<BanRecord>(&mut connection).await
            .map(|rows| rows.into_iter().map(LiveChatBan::from).collect()).map_err(LiveChatError::Database)
    }

    pub async fn active_ban_for(&self, user_id: Option<Uuid>, ip: IpAddr) -> Result<Option<LiveChatBan>, LiveChatError> {
        let mut connection = self.connection().await?; let now = Utc::now(); let network = ipnet::IpNet::from(ip);
        let mut query = live_chat_bans::table.filter(live_chat_bans::expires_at.is_null().or(live_chat_bans::expires_at.gt(now))).into_boxed();
        query = match user_id {
            Some(id) => query.filter(live_chat_bans::user_id.eq(Some(id)).or(live_chat_bans::banned_ip.eq(Some(network)))),
            None => query.filter(live_chat_bans::banned_ip.eq(Some(network))),
        };
        query.order(live_chat_bans::banned_at.desc()).select(BanRecord::as_select())
            .first::<BanRecord>(&mut connection).await.optional()
            .map(|row| row.map(LiveChatBan::from)).map_err(LiveChatError::Database)
    }
}

async fn lock_active_user(connection: &mut AsyncPgConnection, user_id: Uuid) -> Result<(), LiveChatError> {
    let active = users::table.filter(users::user_id.eq(user_id)).filter(users::user_deleted_at.is_null())
        .filter(users::user_hard_purged_at.is_null()).filter(users::user_is_email_verified.eq(true))
        .filter(users::user_is_system_actor.eq(false)).select(users::user_id).for_update()
        .first::<Uuid>(&mut *connection).await.optional()?;
    if active.is_some() { Ok(()) } else { Err(LiveChatError::Unauthorized) }
}

pub(super) async fn lock_active_user_for_call(
    connection: &mut AsyncPgConnection,
    user_id: Uuid,
) -> Result<(), LiveChatError> {
    lock_active_user(connection, user_id).await
}
