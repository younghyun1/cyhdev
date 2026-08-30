mod support;

use std::sync::Arc;

use axum::{
    Extension, Router,
    http::StatusCode,
    middleware::{from_fn, from_fn_with_state},
    routing::get,
};
use reqwest::header::COOKIE;
use uuid::Uuid;

use rust_be_template::{
    features::accounts::domain::{role::RoleType, session::SESSION_COOKIE_NAME},
    routers::middleware::{auth::auth_middleware, role::require_superuser_middleware},
};

use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::{AccountTestContext, VALID_PASSWORD, account_test_context, seed_account},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn middleware_observes_committed_account_and_session_changes() -> TestResult {
    run_database_test(http_authorization_case).await
}

fn http_authorization_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let fixture = seed_account(&context, "HttpBoundary").await?;
        let login = context
            .accounts
            .login(&fixture.email, VALID_PASSWORD, None)
            .await?;

        let protected_router = Router::new()
            .route("/protected", get(protected_handler))
            .layer(from_fn_with_state(
                Arc::clone(&context.sessions),
                auth_middleware,
            ));
        let superuser_router = Router::new()
            .route("/superuser", get(protected_handler))
            .layer(from_fn(require_superuser_middleware))
            .layer(from_fn_with_state(
                Arc::clone(&context.sessions),
                auth_middleware,
            ));
        let router = protected_router.merge(superuser_router);
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let server_task = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _shutdown_result = shutdown_rx.await;
                })
                .await
        });

        let result = exercise_authorization_routes(
            &context,
            &fixture,
            login.session_token.expose(),
            &format!("http://{address}"),
        )
        .await;

        let _shutdown_sent = shutdown_tx.send(()).is_ok();
        let server_result = server_task.await?;
        server_result?;
        result
    })
}

async fn exercise_authorization_routes(
    context: &AccountTestContext,
    fixture: &support::fixtures::AccountFixture,
    session_token: &str,
    base_url: &str,
) -> TestResult {
    let client = reqwest::Client::new();
    require(
        route_status(&client, base_url, "/protected", session_token).await?
            == StatusCode::BAD_REQUEST,
        "unverified session passed authentication middleware",
    )?;

    context
        .accounts
        .verify_email(fixture.verification_token)
        .await?;
    require(
        route_status(&client, base_url, "/protected", session_token).await?
            == StatusCode::NO_CONTENT,
        "verified session was not refreshed for authentication middleware",
    )?;
    require(
        route_status(&client, base_url, "/superuser", session_token).await?
            == StatusCode::FORBIDDEN,
        "user role passed the superuser middleware",
    )?;

    context
        .accounts
        .assign_role(fixture.user_id, RoleType::Younghyun)
        .await?;
    require(
        route_status(&client, base_url, "/superuser", session_token).await?
            == StatusCode::NO_CONTENT,
        "committed superuser role was not refreshed into middleware",
    )?;

    require(
        context.accounts.logout(session_token).await,
        "logout did not revoke the authenticated session",
    )?;
    require(
        route_status(&client, base_url, "/protected", session_token).await?
            == StatusCode::UNAUTHORIZED,
        "revoked session still passed authentication middleware",
    )
}

async fn route_status(
    client: &reqwest::Client,
    base_url: &str,
    path: &str,
    session_token: &str,
) -> TestResult<StatusCode> {
    let response = client
        .get(format!("{base_url}{path}"))
        .header(COOKIE, format!("{SESSION_COOKIE_NAME}={session_token}"))
        .send()
        .await?;
    Ok(response.status())
}

async fn protected_handler(
    Extension(_user_id): Extension<Uuid>,
    Extension(_role_type): Extension<RoleType>,
) -> StatusCode {
    StatusCode::NO_CONTENT
}
