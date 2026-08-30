use std::net::{IpAddr, Ipv4Addr};

use chrono::{Duration, Utc};
use uuid::Uuid;

use super::{
    CachedChatMessage, ChatActor, ChatActorKey, ChatConnectionState, LiveChatCache,
    LiveChatServerEvent, TypingState,
};
use crate::{
    domain::live_chat::{
        message::LIVE_CHAT_SENDER_KIND_USER,
        rtc::{RtcPeerPhase, RtcServerSignal},
    },
    features::accounts::domain::account::DELETED_USER_DISPLAY_NAME,
};

#[tokio::test]
async fn deletion_rewrites_bounded_messages_actors_and_rtc_events() {
    let cache = LiveChatCache::new(1024 * 1024);
    let user_id = Uuid::now_v7();
    let connection_id = Uuid::now_v7();
    let second_connection_id = Uuid::now_v7();
    let actor = ChatActor::user(
        user_id,
        "Original name".to_owned(),
        Some("country".to_owned()),
        Some("https://example.invalid/profile".to_owned()),
    );
    let (disconnect_tx, mut disconnect_rx) = tokio::sync::watch::channel(false);
    let (second_disconnect_tx, mut second_disconnect_rx) = tokio::sync::watch::channel(false);
    assert!(
        cache
            .register_connection(
                connection_id,
                ChatConnectionState {
                    actor: actor.clone(),
                    authority_user_id: Some(user_id),
                    disconnect_tx,
                    room_key: "main".to_owned(),
                    connected_at: Utc::now(),
                },
            )
            .await
    );
    assert!(
        cache
            .register_connection(
                second_connection_id,
                ChatConnectionState {
                    actor: actor.clone(),
                    authority_user_id: Some(user_id),
                    disconnect_tx: second_disconnect_tx,
                    room_key: "main".to_owned(),
                    connected_at: Utc::now(),
                },
            )
            .await
    );
    assert!(
        cache
            .set_typing(TypingState {
                actor: actor.clone(),
                room_key: "main".to_owned(),
                expires_at: Utc::now() + Duration::seconds(5),
            })
            .await
    );

    let message_id = Uuid::now_v7();
    cache
        .append_persisted_chat_message(CachedChatMessage {
            live_chat_message_id: message_id,
            room_key: "main".to_owned(),
            user_id: Some(user_id),
            guest_ip: Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            sender_kind: LIVE_CHAT_SENDER_KIND_USER,
            sender_display_name: "Original name".to_owned(),
            sender_country_flag: Some("country".to_owned()),
            user_profile_picture_url: Some("https://example.invalid/profile".to_owned()),
            message_body: "Retained message".to_owned(),
            message_created_at: Utc::now(),
            message_edited_at: None,
            message_deleted_at: None,
        })
        .await;

    cache.anonymize_deleted_user(user_id).await;

    assert!(cache.is_connected_user_disabled(user_id));
    assert!(disconnect_rx.changed().await.is_ok());
    assert!(*disconnect_rx.borrow_and_update());
    assert!(second_disconnect_rx.changed().await.is_ok());
    assert!(*second_disconnect_rx.borrow_and_update());
    let messages = cache.get_recent_chat_messages(1).await;
    assert_eq!(messages.len(), 1);
    let message = match messages.first() {
        Some(message) => message,
        None => return,
    };
    assert_eq!(message.live_chat_message_id, message_id);
    assert!(message.user_id.is_none());
    assert!(message.guest_ip.is_none());
    assert_eq!(message.sender_display_name, DELETED_USER_DISPLAY_NAME);
    assert!(message.sender_country_flag.is_none());
    assert!(message.user_profile_picture_url.is_none());
    assert_eq!(message.message_body, "Retained message");
    let stats = cache.stats().await;
    assert_eq!(stats.used_bytes, message.estimated_bytes());

    let typing = cache.active_typing_actors(Utc::now()).await;
    assert_eq!(typing.len(), 1);
    let typing_actor = match typing.first() {
        Some(actor) => actor,
        None => return,
    };
    assert_eq!(typing_actor.actor_key, ChatActorKey::User(Uuid::nil()));
    assert!(typing_actor.user_id.is_none());
    assert_eq!(typing_actor.display_name, DELETED_USER_DISPLAY_NAME);
    assert!(typing_actor.country_flag.is_none());
    assert!(typing_actor.user_profile_picture_url.is_none());

    let mut event = LiveChatServerEvent::Rtc(RtcServerSignal::PeerState {
        actor,
        phase: RtcPeerPhase::Joined,
        mic_on: true,
        cam_on: false,
    });
    cache.anonymize_event_for_public(&mut event);
    let LiveChatServerEvent::Rtc(RtcServerSignal::PeerState { actor, .. }) = event else {
        return;
    };
    assert!(actor.user_id.is_none());
    assert_eq!(actor.display_name, DELETED_USER_DISPLAY_NAME);

    cache.unregister_connection(connection_id).await;
    assert!(cache.is_connected_user_disabled(user_id));
    cache.unregister_connection(second_connection_id).await;
    assert!(!cache.is_connected_user_disabled(user_id));
}
