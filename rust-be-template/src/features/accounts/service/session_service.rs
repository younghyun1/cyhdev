//! Fixed-capacity, process-local session authority.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use zeroize::Zeroizing;

use crate::features::accounts::{
    domain::{
        account::{LoginAccount, SessionAccount, SessionPrincipal},
        role::RoleType,
        session::{
            DEFAULT_SESSION_DURATION, SESSION_SECRET_BYTES, Session, SessionKey, SessionToken,
        },
    },
    error::AccountError,
};

pub const MAX_SESSIONS: usize = 16_384;
const MAX_TOKEN_GENERATION_ATTEMPTS: usize = 4;

/// Owns all session authority for the single backend process.
pub struct SessionService {
    sessions: scc::HashMap<SessionKey, Session>,
    active_slots: AtomicUsize,
    max_sessions: usize,
}

impl Default for SessionService {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionService {
    pub fn new() -> Self {
        Self::with_max_sessions(MAX_SESSIONS)
    }

    pub(crate) fn with_max_sessions(max_sessions: usize) -> Self {
        Self {
            sessions: scc::HashMap::with_capacity(max_sessions),
            active_slots: AtomicUsize::new(0),
            max_sessions,
        }
    }

    /// Creates a session and optionally rotates the session presented by this browser.
    pub async fn create(
        &self,
        account: &LoginAccount,
        role_type: RoleType,
        previous_token: Option<&str>,
        valid_for: Option<chrono::Duration>,
    ) -> Result<SessionToken, AccountError> {
        self.create_principal(
            &account.session_principal(),
            role_type,
            previous_token,
            valid_for,
        )
        .await
    }

    /// Creates a session from authority resolved through a non-password login method.
    pub async fn create_principal(
        &self,
        account: &SessionPrincipal,
        role_type: RoleType,
        previous_token: Option<&str>,
        valid_for: Option<chrono::Duration>,
    ) -> Result<SessionToken, AccountError> {
        // Generate before revoking the prior credential so entropy failure cannot log out
        // a browser without replacing its session.
        let (token, key) = self.generate_unique_credential().await?;

        if let Some(previous_token) = previous_token {
            let _ = self.remove(previous_token).await;
        }

        if !self.try_reserve_slot() {
            let _ = self.purge_expired().await;
            if !self.try_reserve_slot() {
                return Err(AccountError::SessionStoreSaturated {
                    max_sessions: self.max_sessions,
                });
            }
        }

        let now = chrono::Utc::now();
        let session = Session {
            is_email_verified: account.is_email_verified,
            created_at: now,
            expires_at: now + valid_for.unwrap_or(DEFAULT_SESSION_DURATION),
            user_id: account.user_id,
            role_type,
            user_language: account.language,
            user_name: Arc::from(account.user_name.as_str()),
            user_country: account.country,
        };

        match self.sessions.insert_async(key, session).await {
            Ok(()) => Ok(token),
            Err(_) => {
                self.release_slots(1);
                Err(AccountError::SessionTokenCollision)
            }
        }
    }

    /// Returns an active session through one hash derivation and one map lookup.
    pub async fn lookup(&self, token: &str) -> Option<Session> {
        let key = SessionKey::from_token(token)?;
        let session = self
            .sessions
            .read_async(&key, |_, session| session.clone())
            .await?;

        if session.is_unexpired_at(chrono::Utc::now()) {
            Some(session)
        } else {
            let _ = self.remove_key(&key).await;
            None
        }
    }

    pub async fn refresh_for_user(
        &self,
        user_id: uuid::Uuid,
        account: &SessionAccount,
        role_type: RoleType,
    ) -> usize {
        let now = chrono::Utc::now();
        let mut refreshed = 0usize;

        self.sessions
            .iter_mut_async(|mut entry| {
                if !entry.is_unexpired_at(now) {
                    let _ = entry.consume();
                    self.release_slots(1);
                } else if entry.user_id == user_id {
                    entry.user_name = Arc::from(account.user_name.as_str());
                    entry.user_country = account.country;
                    entry.user_language = account.language;
                    entry.is_email_verified = account.is_email_verified;
                    entry.role_type = role_type;
                    refreshed += 1;
                }
                true
            })
            .await;

        refreshed
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub async fn remove(&self, token: &str) -> bool {
        match SessionKey::from_token(token) {
            Some(key) => self.remove_key(&key).await,
            None => false,
        }
    }

    pub async fn remove_for_user(&self, user_id: uuid::Uuid) -> usize {
        let mut removed = 0usize;
        self.sessions
            .iter_mut_async(|entry| {
                if entry.user_id == user_id {
                    let _ = entry.consume();
                    self.release_slots(1);
                    removed += 1;
                }
                true
            })
            .await;
        removed
    }

    pub async fn purge_expired(&self) -> (usize, usize) {
        let now = chrono::Utc::now();
        let mut pruned = 0usize;
        self.sessions
            .iter_mut_async(|entry| {
                if !entry.is_unexpired_at(now) {
                    let _ = entry.consume();
                    self.release_slots(1);
                    pruned += 1;
                }
                true
            })
            .await;
        (pruned, self.sessions.len())
    }

    async fn generate_unique_credential(&self) -> Result<(SessionToken, SessionKey), AccountError> {
        for _ in 0..MAX_TOKEN_GENERATION_ATTEMPTS {
            let mut secret = Zeroizing::new([0_u8; SESSION_SECRET_BYTES]);
            getrandom::fill(secret.as_mut()).map_err(AccountError::SessionEntropy)?;
            let key = SessionKey::from_secret(secret.as_ref());
            let key_exists = self.sessions.read_async(&key, |_, _| ()).await.is_some();
            if !key_exists {
                return Ok((SessionToken::from_secret(&secret), key));
            }
        }

        Err(AccountError::SessionTokenCollision)
    }

    async fn remove_key(&self, key: &SessionKey) -> bool {
        match self.sessions.remove_async(key).await {
            Some(_) => {
                self.release_slots(1);
                true
            }
            None => false,
        }
    }

    fn try_reserve_slot(&self) -> bool {
        self.active_slots
            .try_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                (current < self.max_sessions).then_some(current + 1)
            })
            .is_ok()
    }

    fn release_slots(&self, count: usize) {
        self.active_slots.fetch_sub(count, Ordering::AcqRel);
    }
}
