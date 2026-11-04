//! Lets a long-lived connection follow the life of the session that authorized it.

use tokio::sync::watch;

use crate::features::accounts::domain::session::Session;

/// A session as the store holds it.
///
/// The store alone owns the sender, and requests clone only `session`, so removing the
/// entry closes the channel however the session ended and costs requests nothing.
pub(super) struct StoredSession {
    pub(super) session: Session,
    ended: watch::Sender<()>,
}

impl StoredSession {
    pub(super) fn new(session: Session) -> Self {
        Self {
            session,
            ended: watch::channel(()).0,
        }
    }

    pub(super) fn end_signal(&self) -> SessionEnd {
        SessionEnd(self.ended.subscribe())
    }
}

/// Completes once the session it was taken from has ended.
pub struct SessionEnd(watch::Receiver<()>);

impl SessionEnd {
    /// Cancel-safe, so it can sit in a `select!` loop.
    pub async fn ended(&mut self) {
        // Nothing is ever sent, so this returns only when the store drops the sender.
        let _ = self.0.changed().await;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use uuid::Uuid;

    use super::SessionEnd;
    use crate::features::accounts::{
        domain::{account::SessionPrincipal, role::RoleType, session::SessionToken},
        service::session_service::SessionService,
    };

    fn principal(user_id: Uuid) -> SessionPrincipal {
        SessionPrincipal {
            user_id,
            user_name: "session-end-test".to_owned(),
            country: 1,
            language: 1,
            is_email_verified: true,
        }
    }

    async fn signed_in(
        sessions: &SessionService,
        user_id: Uuid,
    ) -> Option<(SessionToken, SessionEnd)> {
        let token = sessions
            .create_principal(&principal(user_id), RoleType::User, None, None)
            .await
            .ok()?;
        let (_, session_end) = sessions.lookup_with_end(token.expose()).await?;
        Some((token, session_end))
    }

    async fn has_ended(session_end: &mut SessionEnd) -> bool {
        tokio::time::timeout(Duration::from_millis(50), session_end.ended())
            .await
            .is_ok()
    }

    #[tokio::test]
    async fn logout_ends_only_that_session() {
        let sessions = SessionService::new();
        let user_id = Uuid::now_v7();
        let Some((token, mut end)) = signed_in(&sessions, user_id).await else {
            panic!("first session was not created");
        };
        let Some((_other_device, mut other_end)) = signed_in(&sessions, user_id).await else {
            panic!("second session was not created");
        };
        assert!(!has_ended(&mut end).await);

        assert!(sessions.remove(token.expose()).await);

        assert!(has_ended(&mut end).await);
        assert!(!has_ended(&mut other_end).await);
    }

    #[tokio::test]
    async fn every_holder_of_a_session_is_told() {
        let sessions = SessionService::new();
        let Some((token, mut first)) = signed_in(&sessions, Uuid::now_v7()).await else {
            panic!("session was not created");
        };
        let Some((_, mut second)) = sessions.lookup_with_end(token.expose()).await else {
            panic!("session was not found");
        };
        assert!(sessions.remove(token.expose()).await);
        assert!(has_ended(&mut first).await);
        assert!(has_ended(&mut second).await);
    }

    #[tokio::test]
    async fn rotation_ends_the_replaced_session() {
        let sessions = SessionService::new();
        let user_id = Uuid::now_v7();
        let Some((token, mut end)) = signed_in(&sessions, user_id).await else {
            panic!("session was not created");
        };
        let rotated = sessions
            .create_principal(
                &principal(user_id),
                RoleType::User,
                Some(token.expose()),
                None,
            )
            .await;
        assert!(rotated.is_ok());
        assert!(has_ended(&mut end).await);
    }

    #[tokio::test]
    async fn revoking_a_user_ends_all_of_their_sessions_and_no_others() {
        let sessions = SessionService::new();
        let user_id = Uuid::now_v7();
        let Some((_, mut first)) = signed_in(&sessions, user_id).await else {
            panic!("first session was not created");
        };
        let Some((_, mut second)) = signed_in(&sessions, user_id).await else {
            panic!("second session was not created");
        };
        let Some((_, mut stranger)) = signed_in(&sessions, Uuid::now_v7()).await else {
            panic!("other user's session was not created");
        };

        assert_eq!(sessions.remove_for_user(user_id).await, 2);

        assert!(has_ended(&mut first).await);
        assert!(has_ended(&mut second).await);
        assert!(!has_ended(&mut stranger).await);
    }

    #[tokio::test]
    async fn the_per_user_cap_ends_the_evicted_session() {
        let sessions = SessionService::with_limits(8, 1);
        let user_id = Uuid::now_v7();
        let Some((_, mut evicted)) = signed_in(&sessions, user_id).await else {
            panic!("first session was not created");
        };
        let Some((_, mut newest)) = signed_in(&sessions, user_id).await else {
            panic!("second session was not created");
        };
        assert!(has_ended(&mut evicted).await);
        assert!(!has_ended(&mut newest).await);
    }

    #[tokio::test]
    async fn expiry_ends_the_session_once_the_purge_reaches_it() {
        let sessions = SessionService::new();
        let Ok(token) = sessions
            .create_principal(
                &principal(Uuid::now_v7()),
                RoleType::User,
                None,
                Some(chrono::Duration::milliseconds(120)),
            )
            .await
        else {
            panic!("session was not created");
        };
        let Some((_, mut end)) = sessions.lookup_with_end(token.expose()).await else {
            panic!("session was not found");
        };
        assert!(!has_ended(&mut end).await);
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(sessions.purge_expired().await.0, 1);
        assert!(has_ended(&mut end).await);
    }

    #[tokio::test]
    async fn no_signal_exists_for_an_unknown_or_expired_session() {
        let sessions = SessionService::new();
        assert!(sessions.lookup_with_end("not-a-token").await.is_none());
        let Ok(token) = sessions
            .create_principal(
                &principal(Uuid::now_v7()),
                RoleType::User,
                None,
                Some(chrono::Duration::milliseconds(-1)),
            )
            .await
        else {
            panic!("session was not created");
        };
        assert!(sessions.lookup_with_end(token.expose()).await.is_none());
        assert_eq!(sessions.len(), 0);
    }
}
