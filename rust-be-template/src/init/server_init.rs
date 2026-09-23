use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use axum_server::{Handle, accept::NoDelayAcceptor, tls_rustls::RustlsConfig};
use lettre::{AsyncSmtpTransport, Tokio1Executor, transport::smtp::authentication::Credentials};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::{
    init::config::EmailConfig, jobs::job_funcs::init_scheduler::task_init,
    routers::main_router::build_router, util::connection_limit::ConnectionLimiter,
};

use super::{
    connection_acceptor::LimitedAcceptor,
    db_config::DbConfig,
    db_pool::{self, DbPoolSettings, build_pool, with_session_options},
    http_redirect::{Ports, redirect_http_to_https},
    http_server::{HttpConnectionLimits, ProtocolTimeouts, configure_protocols},
    shutdown::{DRAIN_DEADLINE, server_shutdown_hooks, wait_for_signal},
    state::ServerState,
};

/// Extra wait beyond the drain deadline for a listener task to report completion.
const LISTENER_EXIT_GRACE: Duration = Duration::from_secs(5);

pub async fn server_init_proc(start: tokio::time::Instant) -> anyhow::Result<()> {
    let host_ip: IpAddr = std::env::var("HOST_IP")
        .map_err(|e| anyhow::anyhow!("Failed to load HOST_IP from .env: {}", e))?
        .parse::<std::net::IpAddr>()
        .map_err(|e| anyhow::anyhow!("Failed to parse HOST_IP as IP address: {}", e))?;

    let host_port: u16 = std::env::var("HOST_PORT")
        .map_err(|e| anyhow::anyhow!("Failed to load HOST_PORT from .env: {}", e))?
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse HOST_PORT as u16: {}", e))?;

    let host_socket_addr: SocketAddr = SocketAddr::new(host_ip, host_port);

    info!(host_socket_addr = %host_socket_addr, "Loaded host configuration.");

    let cert_chain_path: PathBuf = std::env::var("CERT_CHAIN_DIR")
        .map_err(|_| anyhow::anyhow!("CERT_CHAIN_DIR environment variable is not set"))
        .map(PathBuf::from)?;

    let priv_key_path: PathBuf = std::env::var("PRIV_KEY_DIR")
        .map_err(|_| anyhow::anyhow!("PRIV_KEY_DIR environment variable is not set"))
        .map(PathBuf::from)?;

    // configure certificate and private key used by https
    let config = RustlsConfig::from_pem_file(cert_chain_path, priv_key_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to load TLS config: {}", e))?;

    info!(event = "tls_config_loaded", "Loaded TLS configuration");

    let db_url = DbConfig::from_env()
        .map_err(|e| anyhow::anyhow!("Failed to get DB config from environment: {}", e))?
        .to_url()
        .map_err(|e| anyhow::anyhow!("Failed to convert DB config to URL: {}", e))?;

    // Apply embedded migrations before opening the async pool or loading caches,
    // so the schema is guaranteed current. A migration failure is fatal.
    crate::init::db_migrations::run_pending_migrations(db_url.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to apply database migrations: {}", e))?;

    info!(
        event = "database_connect_start",
        "Attempting to connect to database"
    );

    info!(
        event = "database_config_loaded",
        "Loaded database configuration"
    );

    let pool_settings = DbPoolSettings::from_env()?;
    let pool = build_pool(&with_session_options(&db_url)?, pool_settings).await?;

    info!(
        min_idle_connections = pool_settings.min_idle(),
        max_connections = pool_settings.max_size,
        statement_timeout_ms = db_pool::STATEMENT_TIMEOUT.as_millis(),
        lock_timeout_ms = db_pool::LOCK_TIMEOUT.as_millis(),
        idle_in_transaction_timeout_ms = db_pool::IDLE_IN_TRANSACTION_TIMEOUT.as_millis(),
        "Connection pool built"
    );

    let app_name_version: String = std::env::var("APP_NAME_VERSION")
        .map_err(|e| anyhow::anyhow!("Failed to load APP_NAME_VAR from .env: {}", e))?;

    let email_config = EmailConfig::from_env()
        .map_err(|e| anyhow::anyhow!("Failed to load email configs from .env: {}", e))?;
    let email_creds: Credentials = email_config.to_creds();
    let email_client: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay(&email_config.get_url())?
            .credentials(email_creds)
            .build();

    info!(smtp_relay = %email_config.get_url(), "Email client configured");

    let state = Arc::new(
        ServerState::builder()
            .app_name_version(app_name_version)
            .pool(pool)
            .server_start_time(start)
            .email_client(email_client)
            .build()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to build ServerState: {}", e))?,
    );

    // Failures on these should be fatal.
    state.blog_service().synchronize_cache().await;
    state.reference_data_service().synchronize().await?;
    state.i18n_service().synchronize_file_sources().await?;
    state.i18n_service().synchronize_cache().await?;
    state.visitor_service().synchronize_board().await?;
    state
        .wasm_service()
        .synchronize_cache()
        .await
        .map_err(anyhow::Error::from)?;
    state
        .live_chat_service()
        .synchronize_bans()
        .await
        .map_err(anyhow::Error::from)?;
    state
        .live_chat_service()
        .synchronize_messages()
        .await
        .map_err(anyhow::Error::from)?;

    let router = build_router(Arc::clone(&state))?;

    info!(
        event = "server_state_initialized",
        "ServerState initialized"
    );

    // initialize scheduled jobs manager
    task_init(state.clone()).await?;

    // Both listeners share one admission budget because both consume descriptors.
    let limits = HttpConnectionLimits::from_env()?;
    let limiter =
        ConnectionLimiter::new("http_connections", limits.max_total, limits.max_per_client);
    let https_handle = Handle::new();
    let redirect_handle = Handle::new();

    let redirect_task = {
        let (handle, limiter) = (redirect_handle.clone(), limiter.clone());
        let ports = Ports {
            http: 80,
            https: host_port,
        };
        tokio::spawn(async move {
            if let Err(e) = redirect_http_to_https(host_ip, ports, handle, limiter).await {
                error!(error = %e, "HTTP->HTTPS redirect listener exited with error");
            }
        })
    };

    let mut https_server = axum_server::bind_rustls(host_socket_addr, config)
        // HTTP/2 headers and DATA can be separate writes; avoid waiting for delayed TCP ACKs.
        // Admission runs before TCP_NODELAY and TLS so refused sockets cost nothing more.
        .map(|acceptor| acceptor.acceptor(LimitedAcceptor::new(NoDelayAcceptor::new(), limiter)))
        .handle(https_handle.clone());
    configure_protocols(https_server.http_builder(), ProtocolTimeouts::PRODUCTION);

    info!(
        host_port = host_port,
        max_connections = limits.max_total,
        max_connections_per_client = limits.max_per_client,
        "Listening for HTTPS traffic"
    );

    info!(
        elapsed = ?start.elapsed(),
        "Initialization complete; starting server"
    );

    let mut https_task = tokio::spawn(
        https_server.serve(router.into_make_service_with_connect_info::<SocketAddr>()),
    );

    tokio::select! {
        signal = wait_for_signal() => {
            info!(signal, drain_deadline_ms = DRAIN_DEADLINE.as_millis(), "Shutdown requested; draining connections");
        }
        joined = &mut https_task => {
            redirect_handle.shutdown();
            return match joined {
                Ok(Ok(())) => Err(anyhow::anyhow!("HTTPS listener stopped unexpectedly")),
                Ok(Err(e)) => Err(anyhow::anyhow!("Server error: {}", e)),
                Err(e) => Err(anyhow::anyhow!("HTTPS listener task failed: {}", e)),
            };
        }
    }

    https_handle.graceful_shutdown(Some(DRAIN_DEADLINE));
    redirect_handle.graceful_shutdown(Some(DRAIN_DEADLINE));
    await_listener("https", https_task).await;
    await_listener("http_redirect", redirect_task).await;

    let outcomes = server_shutdown_hooks(&state).run().await;
    info!(hooks = outcomes.len(), "Shutdown complete");
    Ok(())
}

/// Waits for a listener to finish draining; axum-server force-closes connections at the
/// drain deadline, so the extra grace only covers task teardown.
async fn await_listener<T>(name: &'static str, task: JoinHandle<T>) {
    match tokio::time::timeout(DRAIN_DEADLINE + LISTENER_EXIT_GRACE, task).await {
        Ok(Ok(_)) => info!(listener = name, "Listener drained"),
        Ok(Err(e)) => error!(listener = name, error = %e, "Listener task failed during shutdown"),
        Err(_) => warn!(
            listener = name,
            "Listener did not stop before the drain deadline"
        ),
    }
}

#[cfg(test)]
#[path = "tls_latency_tests.rs"]
mod tls_latency_tests;
