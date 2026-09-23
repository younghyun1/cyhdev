use std::{error::Error, net::SocketAddr, time::Duration};

use axum::{Router, routing::get};
use axum_server::{Handle, accept::DefaultAcceptor};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    task::JoinHandle,
    time::{Instant, timeout},
};

use super::{HttpConnectionLimits, ProtocolTimeouts, configure_protocols};
use crate::{
    init::connection_acceptor::LimitedAcceptor, util::connection_limit::ConnectionLimiter,
};

type TestResult = Result<(), Box<dyn Error>>;

const SHORT: ProtocolTimeouts = ProtocolTimeouts {
    header_read: Duration::from_millis(200),
    http2_keep_alive_interval: Duration::from_secs(1),
    http2_keep_alive_timeout: Duration::from_secs(1),
};

struct TestServer {
    handle: Handle<SocketAddr>,
    address: SocketAddr,
    task: JoinHandle<std::io::Result<()>>,
}

async fn start(limiter: ConnectionLimiter) -> Result<TestServer, Box<dyn Error>> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
    listener.set_nonblocking(true)?;
    let handle = Handle::new();
    let mut server = axum_server::from_tcp(listener)?
        .acceptor(LimitedAcceptor::new(DefaultAcceptor::new(), limiter))
        .handle(handle.clone());
    configure_protocols(server.http_builder(), SHORT);
    let app = Router::new().route("/", get(|| async { "ok" }));
    let task = tokio::spawn(server.serve(app.into_make_service()));
    let address = handle.listening().await.ok_or("test listener failed")?;
    Ok(TestServer {
        handle,
        address,
        task,
    })
}

async fn read_to_end(stream: &mut TcpStream) -> Result<String, Box<dyn Error>> {
    let mut response = Vec::new();
    timeout(Duration::from_secs(5), stream.read_to_end(&mut response)).await??;
    Ok(String::from_utf8_lossy(&response).into_owned())
}

async fn request(stream: &mut TcpStream, connection: &str) -> Result<String, Box<dyn Error>> {
    let head = format!("GET / HTTP/1.1\r\nHost: localhost\r\nConnection: {connection}\r\n\r\n");
    stream.write_all(head.as_bytes()).await?;
    let mut buffer = vec![0_u8; 1024];
    let read = timeout(Duration::from_secs(5), stream.read(&mut buffer)).await??;
    Ok(String::from_utf8_lossy(&buffer[..read]).into_owned())
}

async fn stop(server: TestServer) -> TestResult {
    server
        .handle
        .graceful_shutdown(Some(Duration::from_millis(100)));
    timeout(Duration::from_secs(5), server.task).await???;
    Ok(())
}

#[tokio::test]
async fn incomplete_request_head_is_closed_after_the_header_timeout() -> TestResult {
    let server = start(ConnectionLimiter::new("test", 8, 8)).await?;
    let mut stream = TcpStream::connect(server.address).await?;
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n")
        .await?;
    let started = Instant::now();
    let response = read_to_end(&mut stream).await?;
    let elapsed = started.elapsed();
    assert!(
        elapsed >= Duration::from_millis(150),
        "closed after {elapsed:?}"
    );
    assert!(
        response.is_empty() || response.starts_with("HTTP/1.1 408"),
        "unexpected response: {response}"
    );
    stop(server).await
}

#[tokio::test]
async fn idle_keep_alive_connection_is_closed_after_the_header_timeout() -> TestResult {
    let server = start(ConnectionLimiter::new("test", 8, 8)).await?;
    let mut stream = TcpStream::connect(server.address).await?;
    assert!(
        request(&mut stream, "keep-alive")
            .await?
            .starts_with("HTTP/1.1 200")
    );
    // No further request: the header timer covers the idle wait for the next head.
    let remainder = read_to_end(&mut stream).await?;
    assert!(
        remainder.is_empty() || remainder.starts_with("HTTP/1.1 408"),
        "unexpected response: {remainder}"
    );
    stop(server).await
}

#[tokio::test]
async fn connection_cap_refuses_excess_connections_from_one_client() -> TestResult {
    let limiter = ConnectionLimiter::new("test", 8, 1);
    let server = start(limiter.clone()).await?;
    let mut admitted = TcpStream::connect(server.address).await?;
    assert!(
        request(&mut admitted, "keep-alive")
            .await?
            .starts_with("HTTP/1.1 200")
    );
    assert_eq!(limiter.active(), 1);

    let mut refused = TcpStream::connect(server.address).await?;
    let refused_response = timeout(Duration::from_secs(5), async {
        let mut buffer = [0_u8; 64];
        refused
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .ok();
        refused.read(&mut buffer).await.unwrap_or(0)
    })
    .await?;
    assert_eq!(
        refused_response, 0,
        "refused connection must close without a response"
    );

    // The admitted connection keeps working, and closing it frees the slot.
    assert!(
        request(&mut admitted, "close")
            .await?
            .starts_with("HTTP/1.1 200")
    );
    drop(admitted);
    let deadline = Instant::now() + Duration::from_secs(5);
    while limiter.active() != 0 && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let mut replacement = TcpStream::connect(server.address).await?;
    assert!(
        request(&mut replacement, "close")
            .await?
            .starts_with("HTTP/1.1 200")
    );
    stop(server).await
}

#[test]
fn connection_limits_default_and_reject_invalid_values() {
    assert_eq!(
        HttpConnectionLimits::parse(None, None).ok(),
        Some(HttpConnectionLimits {
            max_total: 4_096,
            max_per_client: 64
        })
    );
    assert_eq!(
        HttpConnectionLimits::parse(Some("100"), Some(" 10 ")).ok(),
        Some(HttpConnectionLimits {
            max_total: 100,
            max_per_client: 10
        })
    );
    for (total, per_client) in [
        (Some("0"), None),
        (Some("many"), None),
        (None, Some("0")),
        (Some("10"), Some("11")),
        (Some("70000"), None),
    ] {
        assert!(HttpConnectionLimits::parse(total, per_client).is_err());
    }
}
