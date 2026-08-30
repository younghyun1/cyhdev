//! Swagger UI routing and production access control.

use std::sync::Arc;

use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    docs::ApiDoc,
    features::accounts::service::session_service::SessionService,
    init::state::DeploymentEnvironment,
    routers::middleware::{auth::auth_middleware, role::require_superuser_middleware},
};

const SWAGGER_UI_PATH: &str = "/swagger-ui";
const OPENAPI_DOCUMENT_PATH: &str = "/api-docs/openapi.json";

/// Builds the documentation router, protecting both UI assets and the schema in production.
pub(super) fn build_swagger_router(
    sessions: Arc<SessionService>,
    deployment_environment: DeploymentEnvironment,
) -> Router {
    let router = Router::new()
        .merge(SwaggerUi::new(SWAGGER_UI_PATH).url(OPENAPI_DOCUMENT_PATH, ApiDoc::openapi()));

    if deployment_environment == DeploymentEnvironment::Prod {
        return router
            .layer(from_fn(require_superuser_middleware))
            .layer(from_fn_with_state(sessions, auth_middleware));
    }

    router
}

#[cfg(test)]
mod tests {
    use std::{error::Error, net::SocketAddr, sync::Arc};

    use axum::{Router, http::StatusCode};
    use reqwest::{Client, header};
    use serde_json::Value;
    use tokio::{sync::oneshot, task::JoinHandle};
    use uuid::Uuid;

    use crate::features::accounts::{
        domain::{
            account::LoginAccount,
            role::RoleType,
            session::{SESSION_COOKIE_NAME, SessionToken},
        },
        error::AccountError,
        service::session_service::SessionService,
    };

    use super::{OPENAPI_DOCUMENT_PATH, SWAGGER_UI_PATH, build_swagger_router};
    use crate::init::state::DeploymentEnvironment;

    type RunningServer = (
        SocketAddr,
        oneshot::Sender<()>,
        JoinHandle<Result<(), std::io::Error>>,
    );

    async fn serve(router: Router) -> Result<RunningServer, Box<dyn Error>> {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let server = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await
        });
        Ok((address, shutdown_tx, server))
    }

    fn account() -> LoginAccount {
        LoginAccount {
            user_id: Uuid::new_v4(),
            user_name: "swagger-test-user".to_owned(),
            password_hash: "unused-in-swagger-tests".to_owned(),
            is_email_verified: true,
            country: 840,
            language: 1,
        }
    }

    async fn create_session(
        sessions: &SessionService,
        role_type: RoleType,
    ) -> Result<SessionToken, AccountError> {
        sessions.create(&account(), role_type, None, None).await
    }

    fn session_cookie(token: &SessionToken) -> String {
        format!("{SESSION_COOKIE_NAME}={}", token.expose())
    }

    #[tokio::test]
    async fn serves_canonical_ui_assets_and_openapi_document() -> Result<(), Box<dyn Error>> {
        let router =
            build_swagger_router(Arc::new(SessionService::new()), DeploymentEnvironment::Dev);
        let (address, shutdown_tx, server) = serve(router).await?;
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        let origin = format!("http://{address}");

        let redirect = client
            .get(format!("{origin}{SWAGGER_UI_PATH}"))
            .send()
            .await?;
        assert_eq!(redirect.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            redirect.headers().get(header::LOCATION),
            Some(&header::HeaderValue::from_static("/swagger-ui/"))
        );

        let index = client
            .get(format!("{origin}{SWAGGER_UI_PATH}/"))
            .send()
            .await?;
        assert_eq!(index.status(), StatusCode::OK);
        assert!(
            index
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.starts_with("text/html"))
        );
        assert!(index.text().await?.contains("swagger-initializer.js"));

        let initializer = client
            .get(format!("{origin}{SWAGGER_UI_PATH}/swagger-initializer.js"))
            .send()
            .await?;
        assert_eq!(initializer.status(), StatusCode::OK);
        assert!(initializer.text().await?.contains(OPENAPI_DOCUMENT_PATH));

        let stylesheet = client
            .get(format!("{origin}{SWAGGER_UI_PATH}/swagger-ui.css"))
            .send()
            .await?;
        assert_eq!(stylesheet.status(), StatusCode::OK);
        assert!(
            stylesheet
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.starts_with("text/css"))
        );
        assert!(!stylesheet.bytes().await?.is_empty());

        let document = client
            .get(format!("{origin}{OPENAPI_DOCUMENT_PATH}"))
            .send()
            .await?;
        assert_eq!(document.status(), StatusCode::OK);
        let document: Value = serde_json::from_slice(&document.bytes().await?)?;
        assert!(document.get("openapi").and_then(Value::as_str).is_some());
        assert!(document.get("paths").and_then(Value::as_object).is_some());

        let _ = shutdown_tx.send(());
        server.await??;
        Ok(())
    }

    #[tokio::test]
    async fn production_requires_a_superuser_session() -> Result<(), Box<dyn Error>> {
        let sessions = Arc::new(SessionService::new());
        let user_token = create_session(&sessions, RoleType::User).await?;
        let superuser_token = create_session(&sessions, RoleType::Younghyun).await?;
        let router = build_swagger_router(Arc::clone(&sessions), DeploymentEnvironment::Prod);
        let (address, shutdown_tx, server) = serve(router).await?;
        let client = Client::new();
        let origin = format!("http://{address}");

        let unauthenticated = client
            .get(format!("{origin}{SWAGGER_UI_PATH}/"))
            .send()
            .await?;
        assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

        let unauthorized = client
            .get(format!("{origin}{OPENAPI_DOCUMENT_PATH}"))
            .header(header::COOKIE, session_cookie(&user_token))
            .send()
            .await?;
        assert_eq!(unauthorized.status(), StatusCode::FORBIDDEN);

        let index = client
            .get(format!("{origin}{SWAGGER_UI_PATH}/"))
            .header(header::COOKIE, session_cookie(&superuser_token))
            .send()
            .await?;
        assert_eq!(index.status(), StatusCode::OK);

        let document = client
            .get(format!("{origin}{OPENAPI_DOCUMENT_PATH}"))
            .header(header::COOKIE, session_cookie(&superuser_token))
            .send()
            .await?;
        assert_eq!(document.status(), StatusCode::OK);

        let _ = shutdown_tx.send(());
        server.await??;
        Ok(())
    }
}
