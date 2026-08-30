use std::{error::Error, sync::Arc};

use axum::{
    Extension, Router,
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::get,
};
use reqwest::header::COOKIE;
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::{
            account::{LoginAccount, SessionAccount},
            role::RoleType,
            session::SESSION_COOKIE_NAME,
        },
        error::AccountError,
        service::session_service::SessionService,
    },
    routers::middleware::auth::auth_middleware,
};

fn login_account(user_id: Uuid, verified: bool) -> LoginAccount {
    LoginAccount {
        user_id,
        user_name: "session-user".to_owned(),
        password_hash: "unused-in-session-tests".to_owned(),
        is_email_verified: verified,
        country: 840,
        language: 1,
    }
}

#[tokio::test]
async fn capacity_is_strict_and_expired_entries_are_reclaimed() -> Result<(), AccountError> {
    let sessions = SessionService::with_max_sessions(1);
    let first_account = login_account(Uuid::new_v4(), true);
    let second_account = login_account(Uuid::new_v4(), true);
    let first = sessions
        .create(&first_account, RoleType::User, None, None)
        .await?;

    match sessions
        .create(&second_account, RoleType::User, None, None)
        .await
    {
        Err(AccountError::SessionStoreSaturated { max_sessions: 1 }) => {}
        Err(error) => return Err(error),
        Ok(_) => panic!("fixed session capacity admitted an extra session"),
    }
    assert_eq!(sessions.len(), 1);
    assert!(sessions.remove(first.expose()).await);

    let expired = sessions
        .create(
            &first_account,
            RoleType::User,
            None,
            Some(chrono::Duration::seconds(-1)),
        )
        .await?;
    let replacement = sessions
        .create(&second_account, RoleType::User, None, None)
        .await?;

    assert!(sessions.lookup(expired.expose()).await.is_none());
    assert!(sessions.lookup(replacement.expose()).await.is_some());
    assert_eq!(sessions.len(), 1);
    Ok(())
}

#[tokio::test]
async fn rotation_preserves_other_devices_and_user_mutations_propagate() -> Result<(), AccountError> {
    let sessions = SessionService::with_max_sessions(4);
    let user_id = Uuid::new_v4();
    let account = login_account(user_id, false);
    let first = sessions
        .create(&account, RoleType::User, None, None)
        .await?;
    let rotated = sessions
        .create(&account, RoleType::User, Some(first.expose()), None)
        .await?;
    let other_device = sessions
        .create(&account, RoleType::User, None, None)
        .await?;

    assert!(sessions.lookup(first.expose()).await.is_none());
    assert_eq!(sessions.len(), 2);

    let refreshed = sessions
        .refresh_for_user(
            user_id,
            &SessionAccount {
                user_name: "renamed-user".to_owned(),
                is_email_verified: true,
                country: 124,
                language: 2,
            },
            RoleType::Moderator,
        )
        .await;
    assert_eq!(refreshed, 2);
    let session = sessions.lookup(other_device.expose()).await;
    match session {
        Some(session) => {
            assert_eq!(session.user_name.as_ref(), "renamed-user");
            assert!(session.is_email_verified);
            assert_eq!(session.role_type, RoleType::Moderator);
        }
        None => panic!("independent device session disappeared during refresh"),
    }

    assert!(sessions.remove(rotated.expose()).await);
    assert!(sessions.lookup(other_device.expose()).await.is_some());
    assert_eq!(sessions.remove_for_user(user_id).await, 1);
    assert!(sessions.is_empty());
    Ok(())
}

#[tokio::test]
async fn router_accepts_only_an_active_opaque_cookie() -> Result<(), Box<dyn Error>> {
    let sessions = Arc::new(SessionService::with_max_sessions(2));
    let account = login_account(Uuid::new_v4(), true);
    let token = sessions
        .create(&account, RoleType::User, None, None)
        .await?;
    let router = Router::new()
        .route("/protected", get(protected_handler))
        .layer(from_fn_with_state(Arc::clone(&sessions), auth_middleware));
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let address = listener.local_addr()?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
    });
    let client = reqwest::Client::new();
    let url = format!("http://{address}/protected");

    let valid = client
        .get(&url)
        .header(COOKIE, format!("{SESSION_COOKIE_NAME}={}", token.expose()))
        .send()
        .await?;
    assert_eq!(valid.status(), StatusCode::NO_CONTENT);

    let malformed = client
        .get(&url)
        .header(COOKIE, format!("{SESSION_COOKIE_NAME}={}", "!".repeat(43)))
        .send()
        .await?;
    assert_eq!(malformed.status(), StatusCode::UNAUTHORIZED);

    assert!(sessions.remove(token.expose()).await);
    let revoked = client
        .get(&url)
        .header(COOKIE, format!("{SESSION_COOKIE_NAME}={}", token.expose()))
        .send()
        .await?;
    assert_eq!(revoked.status(), StatusCode::UNAUTHORIZED);

    let _ = shutdown_tx.send(());
    server.await??;
    Ok(())
}

async fn protected_handler(
    Extension(_user_id): Extension<Uuid>,
    Extension(_role_type): Extension<RoleType>,
) -> StatusCode {
    StatusCode::NO_CONTENT
}
