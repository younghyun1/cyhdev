//! Synthetic retained identities exercise due delivery and purge without waiting ninety days.
use diesel::sql_query;
use diesel::sql_types::{Integer, Uuid as SqlUuid};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

pub const DELETED: Uuid = Uuid::from_u128(0x01990000000070008000000000000004);
pub const NOTICE: Uuid = Uuid::from_u128(0x01990000000070008000000000000050);
pub const PURGE: Uuid = Uuid::from_u128(0x01990000000070008000000000000005);

pub async fn seed(
    connection: &mut AsyncPgConnection,
    country: i32,
    language: i32,
) -> anyhow::Result<()> {
    for (id, age) in [(DELETED, 88), (PURGE, 100)] {
        sql_query("INSERT INTO users (user_id,user_name,user_email,user_password_hash,user_is_email_verified,user_country,user_language,user_created_at,user_updated_at,user_deleted_at,user_purge_after) VALUES ($1,$6,$5,'disabled',false,$2,$3,now()-$4*INTERVAL '1 day',now()-$4*INTERVAL '1 day',now()-$4*INTERVAL '1 day',now()+(90-$4)*INTERVAL '1 day')")
            .bind::<SqlUuid,_>(id).bind::<Integer,_>(country).bind::<Integer,_>(language).bind::<Integer,_>(age)
            .bind::<diesel::sql_types::Text,_>(format!("deleted-{id}@example.test"))
            .bind::<diesel::sql_types::Text,_>(format!("Deleted{id}")).execute(connection).await?;
        sql_query("INSERT INTO deleted_account_retention (deleted_account_retention_user_id,deleted_account_retention_user_name,deleted_account_retention_email,deleted_account_retention_country,deleted_account_retention_language,deleted_account_retention_created_at) VALUES ($1,$5,$6,$2,$3,now()-$4*INTERVAL '1 day')")
            .bind::<SqlUuid,_>(id).bind::<Integer,_>(country).bind::<Integer,_>(language).bind::<Integer,_>(age)
            .bind::<diesel::sql_types::Text,_>(format!("Fixture{id}"))
            .bind::<diesel::sql_types::Text,_>(if id==DELETED {"fixture-retained@example.test"}else{"fixture-purge@example.test"}).execute(connection).await?;
    }
    sql_query("INSERT INTO account_retention_notifications (account_retention_notification_id,account_retention_notification_user_id,account_retention_notification_stage,account_retention_notification_scheduled_for,account_retention_notification_next_attempt_at) SELECT $1,user_id,'seven_days_before_purge',user_purge_after-INTERVAL '7 days',user_purge_after-INTERVAL '7 days' FROM users WHERE user_id=$2")
        .bind::<SqlUuid,_>(NOTICE).bind::<SqlUuid,_>(DELETED).execute(connection).await?;
    // Retry makes this due explicitly after the census; the scheduler cannot consume it early.
    sql_query("UPDATE account_retention_notifications SET account_retention_notification_next_attempt_at=now()+INTERVAL '2 days',account_retention_notification_updated_at=now() WHERE account_retention_notification_id=$1")
        .bind::<SqlUuid,_>(NOTICE).execute(connection).await?;
    Ok(())
}
