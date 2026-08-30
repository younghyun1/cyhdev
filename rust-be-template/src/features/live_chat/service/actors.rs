use std::net::IpAddr;

use uuid::Uuid;

use super::{
    super::{domain::actor::ChatActor, error::LiveChatError},
    live_chat_service::LiveChatService,
};

impl LiveChatService {
    pub async fn user_actor(
        &self,
        user_id: Uuid,
        display_name: String,
        country_code: i32,
    ) -> Result<ChatActor, LiveChatError> {
        let presentation = self.repository.user_presentation(&[user_id]).await?;
        let flag = self.country_flags.country_flag(country_code).await;
        let profile = presentation.profile_urls.get(&user_id).cloned();
        Ok(ChatActor::user(user_id, display_name, flag, profile))
    }

    pub async fn guest_actor(&self, ip: IpAddr) -> ChatActor {
        let flag = match self.geo_ip.country_alpha2(ip) {
            Some(code) => self.alpha2_flags.flag(&code).await,
            None => None,
        };
        ChatActor::guest(ip, flag)
    }
}
