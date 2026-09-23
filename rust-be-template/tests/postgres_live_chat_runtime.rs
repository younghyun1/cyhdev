//! Live-chat persistence behind runtime limits: expiring prefix bans, room-scoped
//! history, tied keyset cursors, and call-row reconciliation.

mod support;

use std::net::IpAddr;

use chrono::{Duration, Utc};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use rust_be_template::{
    features::live_chat::{
        domain::{
            actor::ChatActor, guest_identity::GuestIdentityKey, ip_prefix::LiveChatIpPrefix,
            message::LIVE_CHAT_SENDER_KIND_GUEST,
        },
        repository::{
            compatibility::LiveChatMessageInsertable, live_chat_repository::LiveChatRepository,
        },
    },
    schema::{live_chat_call_participants, live_chat_calls, live_chat_messages},
};
use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::account_test_context,
};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn abuse_bans_cover_the_ipv6_prefix_and_expire() -> TestResult {
    run_database_test(ban_case).await
}

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn history_is_room_scoped_and_tied_cursors_are_exact() -> TestResult {
    run_database_test(history_case).await
}

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn open_calls_and_participants_close_idempotently() -> TestResult {
    run_database_test(call_case).await
}

fn guest(ip: &str) -> TestResult<ChatActor> {
    Ok(ChatActor::guest(
        ip.parse()?,
        &GuestIdentityKey::from_secret(&[0x33; 32]),
        None,
    ))
}

fn ban_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let repository = LiveChatRepository::new(context.pool.clone());
        let sender: IpAddr = "2001:db8:aa:bb:1::1".parse()?;
        let network = LiveChatIpPrefix::of(sender).network();
        let ban = repository
            .insert_abuse_ban(
                &guest("2001:db8:aa:bb:1::1")?,
                network,
                Utc::now() + Duration::hours(24),
            )
            .await?;
        require(
            ban.expires_at.is_some(),
            "automatic ban was stored without expiry",
        )?;
        require(
            ban.banned_ip.map(|stored| stored.to_string()).as_deref()
                == Some("2001:db8:aa:bb::/64"),
            "automatic ban did not store the /64 network",
        )?;

        let neighbor: IpAddr = "2001:db8:aa:bb:ffff::2".parse()?;
        let outsider: IpAddr = "2001:db8:aa:bc::1".parse()?;
        require(
            repository.active_ban_for(None, neighbor).await?.is_some(),
            "prefix ban missed a neighbor in the same /64",
        )?;
        require(
            repository.active_ban_for(None, outsider).await?.is_none(),
            "prefix ban leaked outside its /64",
        )?;

        let expired: IpAddr = "198.51.100.20".parse()?;
        repository
            .insert_abuse_ban(
                &guest("198.51.100.20")?,
                LiveChatIpPrefix::of(expired).network(),
                Utc::now() - Duration::seconds(1),
            )
            .await?;
        require(
            repository.active_ban_for(None, expired).await?.is_none(),
            "expired automatic ban still applied",
        )?;
        require(
            repository.active_bans(100).await?.len() == 1,
            "active ban listing included an expired ban",
        )
    })
}

fn history_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let repository = LiveChatRepository::new(context.pool.clone());
        let tied_at = Utc::now() - Duration::minutes(5);
        // Ascending UUIDs sharing one timestamp exercise the id tie-breaker.
        let tied_ids = (1..=5_u128)
            .map(|index| Uuid::from_u128((0x0190_0000_0000_7000_8000_0000_0000_0000) + index))
            .collect::<Vec<_>>();
        let mut connection = context.pool.get().await?;
        for id in &tied_ids {
            insert_guest_message(&mut connection, *id, "main", tied_at).await?;
        }
        insert_guest_message(&mut connection, Uuid::now_v7(), "side", Utc::now()).await?;
        let deleted_id = tied_ids[2];
        diesel::update(live_chat_messages::table.find(deleted_id))
            .set(live_chat_messages::message_deleted_at.eq(Some(Utc::now())))
            .execute(&mut connection)
            .await?;
        drop(connection);

        let recent = repository.recent_messages("main", 10).await?;
        require(
            recent.iter().all(|message| message.room_key == "main"),
            "recent history included another room",
        )?;
        let recent_ids = recent
            .iter()
            .map(|message| message.live_chat_message_id)
            .collect::<Vec<_>>();
        require(
            recent_ids == [tied_ids[0], tied_ids[1], tied_ids[3], tied_ids[4]],
            "recent history lost tie order or returned a tombstone",
        )?;

        let first_page = repository.messages_before(tied_ids[4], 2).await?;
        let first_ids = first_page
            .iter()
            .map(|message| message.live_chat_message_id)
            .collect::<Vec<_>>();
        require(
            first_ids == [tied_ids[1], tied_ids[3]],
            "tied cursor page skipped or repeated rows",
        )?;
        let second_page = repository.messages_before(tied_ids[1], 2).await?;
        require(
            second_page.len() == 1 && second_page[0].live_chat_message_id == tied_ids[0],
            "second tied page was not exact",
        )?;
        let from_tombstone = repository.messages_before(deleted_id, 10).await?;
        let tombstone_ids = from_tombstone
            .iter()
            .map(|message| message.live_chat_message_id)
            .collect::<Vec<_>>();
        require(
            tombstone_ids == [tied_ids[0], tied_ids[1]],
            "tombstone cursor did not page older rows",
        )
    })
}

async fn insert_guest_message(
    connection: &mut diesel_async::AsyncPgConnection,
    id: Uuid,
    room_key: &str,
    created_at: chrono::DateTime<Utc>,
) -> TestResult {
    diesel::insert_into(live_chat_messages::table)
        .values(LiveChatMessageInsertable {
            live_chat_message_id: id,
            room_key: room_key.to_owned(),
            user_id: None,
            guest_ip: Some(ipnet::IpNet::from("192.0.2.10".parse::<IpAddr>()?)),
            sender_kind: LIVE_CHAT_SENDER_KIND_GUEST,
            sender_display_name: "tied guest".to_owned(),
            message_body: format!("message {id}"),
            message_created_at: created_at,
        })
        .execute(connection)
        .await?;
    Ok(())
}

fn call_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let repository = LiveChatRepository::new(context.pool.clone());
        let call_id = repository.open_call("main", None).await?;
        let first = guest("192.0.2.30")?;
        let second = guest("192.0.2.31")?;
        let left_participant = repository.join_call(call_id, &first, true, true).await?;
        repository.join_call(call_id, &second, true, false).await?;
        repository.leave_call(left_participant).await?;

        require(
            repository.close_open_calls().await? == (1, 1),
            "reconciliation did not close exactly the open rows",
        )?;
        require(
            repository.close_open_calls().await? == (0, 0),
            "reconciliation was not idempotent",
        )?;

        let mut connection = context.pool.get().await?;
        let open_calls = live_chat_calls::table
            .filter(live_chat_calls::call_ended_at.is_null())
            .count()
            .get_result::<i64>(&mut connection)
            .await?;
        let open_participants = live_chat_call_participants::table
            .filter(live_chat_call_participants::participant_left_at.is_null())
            .count()
            .get_result::<i64>(&mut connection)
            .await?;
        require(
            open_calls == 0 && open_participants == 0,
            "call rows remained open after reconciliation",
        )
    })
}
