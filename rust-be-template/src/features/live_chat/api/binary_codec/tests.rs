use std::{net::IpAddr, str::FromStr};

use chrono::{TimeZone, Utc};
use uuid::Uuid;

use super::*;
use crate::features::live_chat::{
    domain::{
        actor::ChatActor, guest_identity::GuestIdentityKey, message::LIVE_CHAT_SENDER_KIND_GUEST,
    },
    service::cache::{CachedChatMessage, LiveChatServerEvent},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const CLIENT_MESSAGE_ID: &str = "018f3f7d-5a76-7d8f-8123-456789abcdef";
const MESSAGE_ID: &str = "018f3f7d-5a76-7d8f-8123-456789abc001";

#[test]
fn decodes_send_message_client_frame() -> TestResult {
    let mut frame = Vec::new();
    frame.push(CLIENT_SEND_MESSAGE);
    frame.extend_from_slice(Uuid::parse_str(CLIENT_MESSAGE_ID)?.as_bytes());
    frame.extend_from_slice(&5u16.to_be_bytes());
    frame.extend_from_slice("hello".as_bytes());

    match decode_client_event(&frame)? {
        LiveChatBinaryClientEvent::SendMessage {
            client_message_id,
            body,
        } => {
            assert_eq!(client_message_id, CLIENT_MESSAGE_ID);
            assert_eq!(body, "hello");
            Ok(())
        }
        _ => Err("expected send message event".into()),
    }
}

#[test]
fn decodes_typing_and_ping_client_frames() -> TestResult {
    let start = decode_client_event(&[CLIENT_TYPING_START])?;
    let stop = decode_client_event(&[CLIENT_TYPING_STOP])?;
    let mut ping_frame = vec![CLIENT_PING];
    ping_frame.extend_from_slice(&42u64.to_be_bytes());
    let ping = decode_client_event(&ping_frame)?;

    assert!(matches!(
        start,
        LiveChatBinaryClientEvent::Typing { is_typing: true }
    ));
    assert!(matches!(
        stop,
        LiveChatBinaryClientEvent::Typing { is_typing: false }
    ));
    assert!(matches!(
        ping,
        LiveChatBinaryClientEvent::Heartbeat { nonce } if nonce == "42"
    ));
    Ok(())
}

#[test]
fn rejects_trailing_and_truncated_client_frames() {
    assert!(decode_client_event(&[CLIENT_TYPING_START, 0]).is_err());
    assert!(decode_client_event(&[CLIENT_SEND_MESSAGE, 1, 2, 3]).is_err());
    assert!(decode_client_event(&[0xff]).is_err());
}

#[test]
fn encodes_presence_and_error_server_frames() -> TestResult {
    let presence = encode_server_event(&LiveChatServerEvent::Presence {
        connected_count: 513,
    })?;
    assert_eq!(presence, vec![SERVER_PRESENCE, 0, 0, 2, 1]);

    let error = encode_server_event(&LiveChatServerEvent::Error {
        code: "bad".to_string(),
        message: "nope".to_string(),
    })?;
    assert_eq!(error[0], SERVER_ERROR);
    assert_eq!(&error[1..3], &3u16.to_be_bytes());
    assert_eq!(&error[3..6], b"bad");
    assert_eq!(&error[6..8], &4u16.to_be_bytes());
    assert_eq!(&error[8..12], b"nope");
    Ok(())
}

#[test]
fn guest_messages_carry_no_address_bytes() -> TestResult {
    for address in ["203.0.113.9", "2001:db8::1"] {
        let guest_ip = IpAddr::from_str(address)?;
        let frame = encode_server_event(&LiveChatServerEvent::Message {
            message: cached_guest_message(guest_ip)?,
        })?;
        assert!(!contains_subsequence(&frame, &address_octets(guest_ip)));
        // Opcode, message id, room string, then the payload-free guest tag.
        let actor_offset = 1 + 16 + 2 + "main".len();
        assert_eq!(frame[actor_offset], ACTOR_GUEST);
        assert_eq!(frame[actor_offset + 1], LIVE_CHAT_SENDER_KIND_GUEST as u8);
    }
    Ok(())
}

#[test]
fn encodes_hello_with_opaque_guest_key_and_recent_messages() -> TestResult {
    let guest_ip = IpAddr::from_str("198.51.100.7")?;
    let key = test_key();
    let actor = ChatActor::guest(guest_ip, &key, Some("🇺🇸".to_string()));
    let guest_key = key.identify(guest_ip).actor_key;
    let frame = encode_server_event(&LiveChatServerEvent::Hello {
        actor,
        recent_messages: vec![cached_guest_message(guest_ip)?],
        connected_count: 3,
    })?;

    assert_eq!(frame[0], SERVER_HELLO);
    assert_eq!(frame[1], ACTOR_GUEST);
    assert_eq!(&frame[2..4], &(guest_key.len() as u16).to_be_bytes());
    assert_eq!(&frame[4..4 + guest_key.len()], guest_key.as_bytes());
    assert!(!contains_subsequence(&frame, &address_octets(guest_ip)));
    assert!(!contains_subsequence(&frame, b"198.51.100.7"));
    assert!(contains_subsequence(&frame, &3u32.to_be_bytes()));
    assert!(contains_subsequence(&frame, &1u16.to_be_bytes()));
    Ok(())
}

#[test]
fn json_actor_and_message_omit_guest_addresses() -> TestResult {
    let guest_ip = IpAddr::from_str("192.0.2.44")?;
    let actor = ChatActor::guest(guest_ip, &test_key(), None);
    let actor_json = serde_json::to_value(&actor)?;
    assert!(actor_json.get("guest_ip").is_none());
    assert_eq!(actor_json["actor_key"]["type"], "guest");
    let message_json = serde_json::to_string(&cached_guest_message(guest_ip)?)?;
    assert!(!message_json.contains("192.0.2.44"));
    assert!(!message_json.contains("guest_ip"));
    Ok(())
}

fn test_key() -> GuestIdentityKey {
    GuestIdentityKey::from_secret(&[0x42; 32])
}

fn cached_guest_message(guest_ip: IpAddr) -> Result<CachedChatMessage, Box<dyn std::error::Error>> {
    Ok(CachedChatMessage {
        live_chat_message_id: Uuid::parse_str(MESSAGE_ID)?,
        room_key: "main".to_string(),
        user_id: None,
        guest_ip: Some(guest_ip),
        sender_kind: LIVE_CHAT_SENDER_KIND_GUEST,
        sender_display_name: test_key().identify(guest_ip).nickname,
        sender_country_flag: Some("🇺🇸".to_string()),
        user_profile_picture_url: None,
        message_body: "hello".to_string(),
        message_created_at: Utc
            .timestamp_millis_opt(1_700_000_000_000)
            .single()
            .ok_or("valid timestamp expected")?,
        message_edited_at: None,
        message_deleted_at: None,
    })
}

fn address_octets(ip: IpAddr) -> Vec<u8> {
    match ip {
        IpAddr::V4(ip) => ip.octets().to_vec(),
        IpAddr::V6(ip) => ip.octets().to_vec(),
    }
}

fn contains_subsequence(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}
