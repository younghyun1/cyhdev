use std::net::IpAddr;

use uuid::Uuid;

use crate::features::accounts::domain::account::DELETED_USER_DISPLAY_NAME;
use crate::features::live_chat::domain::{
    guest_nickname::guest_nickname_for_ip,
    message::{LIVE_CHAT_SENDER_KIND_GUEST, LIVE_CHAT_SENDER_KIND_USER},
};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum ChatActorKey {
    User(Uuid),
    Guest(String),
}

#[derive(Debug, Clone)]
pub struct ChatActor {
    pub actor_key: ChatActorKey,
    pub sender_kind: i16,
    pub user_id: Option<Uuid>,
    pub guest_ip: Option<IpAddr>,
    pub display_name: String,
    pub country_flag: Option<String>,
    pub user_profile_picture_url: Option<String>,
}

impl ChatActor {
    pub fn guest(ip: IpAddr, country_flag: Option<String>) -> Self {
        let display_name = guest_nickname_for_ip(ip);
        Self {
            actor_key: ChatActorKey::Guest(ip.to_string()),
            sender_kind: LIVE_CHAT_SENDER_KIND_GUEST,
            user_id: None,
            guest_ip: Some(ip),
            display_name,
            country_flag,
            user_profile_picture_url: None,
        }
    }

    pub fn user(
        user_id: Uuid,
        display_name: String,
        country_flag: Option<String>,
        user_profile_picture_url: Option<String>,
    ) -> Self {
        Self {
            actor_key: ChatActorKey::User(user_id),
            sender_kind: LIVE_CHAT_SENDER_KIND_USER,
            user_id: Some(user_id),
            guest_ip: None,
            display_name,
            country_flag,
            user_profile_picture_url,
        }
    }

    /// Remove every public identity field while retaining the user sender kind.
    pub fn anonymize_deleted_user(&mut self, deleted_user_id: Uuid) -> bool {
        if self.user_id != Some(deleted_user_id) {
            return false;
        }
        self.actor_key = ChatActorKey::User(Uuid::nil());
        self.user_id = None;
        self.guest_ip = None;
        self.display_name = DELETED_USER_DISPLAY_NAME.to_owned();
        self.country_flag = None;
        self.user_profile_picture_url = None;
        true
    }
}
