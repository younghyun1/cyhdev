use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::schema::{live_chat_call_participants, live_chat_calls};

use super::{
    live_chat_repository::LiveChatRepository,
    messages::lock_active_user_for_call,
    records::{NewCallRecord, NewParticipantRecord},
    super::{domain::actor::ChatActor, error::LiveChatError},
};

impl LiveChatRepository {
    pub async fn open_call(&self, room_key: &str, user_id: Option<Uuid>) -> Result<Uuid, LiveChatError> {
        let mut connection = self.connection().await?; let call_id = Uuid::now_v7();
        connection.transaction::<Uuid, LiveChatError, _>(async move |connection| {
            if let Some(user_id) = user_id { lock_active_user_for_call(connection, user_id).await?; }
            diesel::insert_into(live_chat_calls::table).values(NewCallRecord {
                live_chat_call_id: call_id, room_key: room_key.to_owned(), call_started_at: Utc::now(),
            }).execute(&mut *connection).await?; Ok(call_id)
        }).await
    }

    pub async fn close_call(&self, call_id: Uuid) -> Result<(), LiveChatError> {
        let mut connection = self.connection().await?;
        diesel::update(live_chat_calls::table.filter(live_chat_calls::live_chat_call_id.eq(call_id)
            .and(live_chat_calls::call_ended_at.is_null())))
            .set(live_chat_calls::call_ended_at.eq(Utc::now())).execute(&mut connection).await?;
        Ok(())
    }

    pub async fn join_call(
        &self, call_id: Uuid, actor: &ChatActor, audio: bool, video: bool,
    ) -> Result<Uuid, LiveChatError> {
        let mut connection = self.connection().await?; let participant_id = Uuid::now_v7();
        connection.transaction::<Uuid, LiveChatError, _>(async move |connection| {
            if let Some(user_id) = actor.user_id { lock_active_user_for_call(connection, user_id).await?; }
            diesel::insert_into(live_chat_call_participants::table).values(NewParticipantRecord {
                live_chat_call_participant_id: participant_id, live_chat_call_id: call_id,
                user_id: actor.user_id, guest_ip: actor.guest_ip.map(ipnet::IpNet::from),
                participant_sender_kind: actor.sender_kind, participant_display_name: actor.display_name.clone(),
                participant_joined_at: Utc::now(), participant_had_audio: audio, participant_had_video: video,
            }).execute(&mut *connection).await?; Ok(participant_id)
        }).await
    }

    pub async fn leave_call(&self, participant_id: Uuid) -> Result<(), LiveChatError> {
        let mut connection = self.connection().await?;
        diesel::update(live_chat_call_participants::table.filter(
            live_chat_call_participants::live_chat_call_participant_id.eq(participant_id)
                .and(live_chat_call_participants::participant_left_at.is_null())))
            .set(live_chat_call_participants::participant_left_at.eq(Utc::now()))
            .execute(&mut connection).await?; Ok(())
    }
}
