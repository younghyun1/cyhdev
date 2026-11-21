//! A server with the archive routes behind the guards the root router applies.

use std::{net::SocketAddr, path::Path, sync::Arc};

use axum::{
    Extension,
    http::{StatusCode, header},
    middleware::{from_fn, from_fn_with_state},
};
use kcna_archive_core::service::archive::Archive;
use serde_json::Value;
use tokio::task::JoinHandle;
use uuid::Uuid;

use super::routes::{admin_routes, reader_routes};
use crate::{
    features::{
        accounts::{
            domain::{account::LoginAccount, role::RoleType, session::SESSION_COOKIE_NAME},
            service::session_service::SessionService,
        },
        kcna_archive::service::{
            archive_service::KcnaArchiveService, state::ServiceState, test_store::MemoryReaderStore,
        },
    },
    init::state::{DeploymentEnvironment, PublicAppOrigin},
    routers::middleware::{
        auth::auth_middleware,
        role::require_superuser_middleware,
        sensitive_response::{sensitive_admin_response_headers, sensitive_response_headers},
        trusted_origin::{TrustedOrigins, require_trusted_origin},
    },
};

pub(super) const TRUSTED_ORIGIN: &str = "https://cyhdev.com";

/// One signed-in account: its identifier and its session token.
pub(super) struct Caller {
    pub(super) id: Uuid,
    pub(super) token: String,
}

/// A caller of each kind the gate distinguishes.
pub(super) struct Harness {
    address: SocketAddr,
    pub(super) admin: Caller,
    pub(super) user: Caller,
    pub(super) moderator: Caller,
    /// A member whose address is not verified.
    pub(super) unverified: Caller,
    pub(super) store: Arc<MemoryReaderStore>,
    server: JoinHandle<std::io::Result<()>>,
    client: reqwest::Client,
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.server.abort();
    }
}

async fn caller(
    sessions: &SessionService,
    role: RoleType,
    is_email_verified: bool,
) -> anyhow::Result<Caller> {
    let account = LoginAccount {
        user_id: Uuid::new_v4(),
        user_name: "archive-test".into(),
        password_hash: "unused".into(),
        is_email_verified,
        country: 840,
        language: 1,
    };
    let token = sessions.create(&account, role, None, None).await?;
    Ok(Caller {
        id: account.user_id,
        token: token.expose().to_owned(),
    })
}

pub(super) async fn harness(directory: &Path) -> anyhow::Result<Harness> {
    let sessions = Arc::new(SessionService::new());
    let admin = caller(&sessions, RoleType::Younghyun, true).await?;
    let user = caller(&sessions, RoleType::User, true).await?;
    let moderator = caller(&sessions, RoleType::Moderator, true).await?;
    let unverified = caller(&sessions, RoleType::User, false).await?;
    // The accounts the store knows as active; any other identifier is unknown.
    let store = Arc::new(MemoryReaderStore::with_accounts([
        (admin.id, "admin"),
        (user.id, "member"),
        (moderator.id, "moderator"),
        (unverified.id, "unverified"),
    ]));
    let service = Arc::new(
        KcnaArchiveService::start_with_store(
            directory.to_owned(),
            Arc::new(Archive::load_into_memory),
            Arc::clone(&store) as _,
        )
        .await,
    );
    // Wait for the first load so every test starts from a settled state.
    for _ in 0..1000 {
        if service.status().await?.state != ServiceState::Loading {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let origin = PublicAppOrigin::parse(Some(TRUSTED_ORIGIN), DeploymentEnvironment::Prod)?;
    let origins = Arc::new(TrustedOrigins::from_environment(
        DeploymentEnvironment::Prod,
        &origin,
    )?);
    let readers = reader_routes::<()>()
        .layer(Extension(Arc::clone(&service)))
        .layer(from_fn_with_state(Arc::clone(&sessions), auth_middleware))
        .layer(from_fn(sensitive_response_headers));
    let administrators = admin_routes::<()>()
        .layer(Extension(service))
        .layer(from_fn(require_superuser_middleware))
        .layer(from_fn_with_state(sessions, auth_middleware));
    let router = readers
        .merge(administrators)
        .layer(from_fn_with_state(origins, require_trusted_origin))
        .layer(from_fn(sensitive_admin_response_headers));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let server = tokio::spawn(async move { axum::serve(listener, router).await });
    Ok(Harness {
        address,
        admin,
        user,
        moderator,
        unverified,
        store,
        server,
        client: reqwest::Client::new(),
    })
}

impl Harness {
    pub(super) fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        token: Option<&str>,
        origin: Option<&str>,
    ) -> reqwest::RequestBuilder {
        let mut request = self
            .client
            .request(method, format!("http://{}{path}", self.address));
        if let Some(token) = token {
            request = request.header(header::COOKIE, format!("{SESSION_COOKIE_NAME}={token}"));
        }
        if let Some(origin) = origin {
            request = request.header(header::ORIGIN, origin);
        }
        request
    }

    /// A read as `token`, with the status and the parsed body.
    pub(super) async fn get_as(
        &self,
        token: &str,
        path: &str,
    ) -> anyhow::Result<(StatusCode, Value)> {
        let response = self
            .request(reqwest::Method::GET, path, Some(token), None)
            .send()
            .await?;
        let status = response.status();
        Ok((status, serde_json::from_str(&response.text().await?)?))
    }

    /// A read as the superuser.
    pub(super) async fn get_json(&self, path: &str) -> anyhow::Result<(StatusCode, Value)> {
        self.get_as(&self.admin.token, path).await
    }

    /// Adds `reader` to the list as `token`, from the trusted origin.
    pub(super) async fn grant(
        &self,
        token: &str,
        reader: Uuid,
    ) -> anyhow::Result<(StatusCode, Value)> {
        let response = self
            .request(
                reqwest::Method::POST,
                "/api/admin/kcna-archive/readers",
                Some(token),
                Some(TRUSTED_ORIGIN),
            )
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::json!({ "userId": reader }).to_string())
            .send()
            .await?;
        let status = response.status();
        Ok((status, serde_json::from_str(&response.text().await?)?))
    }

    /// Removes `reader` from the list as `token`, from the trusted origin.
    pub(super) async fn revoke(
        &self,
        token: &str,
        reader: Uuid,
    ) -> anyhow::Result<(StatusCode, Value)> {
        let response = self
            .request(
                reqwest::Method::DELETE,
                &format!("/api/admin/kcna-archive/readers/{reader}"),
                Some(token),
                Some(TRUSTED_ORIGIN),
            )
            .send()
            .await?;
        let status = response.status();
        Ok((status, serde_json::from_str(&response.text().await?)?))
    }
}
