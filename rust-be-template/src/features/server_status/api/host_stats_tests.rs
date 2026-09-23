use std::{error::Error, time::Duration};

use axum::{Router, body::Bytes, extract::ws::WebSocketUpgrade, routing::get};
use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, time::timeout};
use tokio_tungstenite::{WebSocketStream, client_async, tungstenite::Message};

use super::{SocketTiming, bounded_upgrade, stream_samples};

type TestResult = Result<(), Box<dyn Error>>;

const FAST: SocketTiming = SocketTiming {
    sample_interval: Duration::from_millis(20),
    ping_interval: Duration::from_millis(50),
    idle_timeout: Duration::from_millis(200),
    write_timeout: Duration::from_millis(500),
};

async fn connect() -> Result<WebSocketStream<TcpStream>, Box<dyn Error>> {
    let app = Router::new().route(
        "/ws",
        get(|websocket: WebSocketUpgrade| async move {
            bounded_upgrade(websocket).on_upgrade(|socket| {
                stream_samples(socket, FAST, || async { Bytes::from_static(&[7; 20]) })
            })
        }),
    );
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let address = listener.local_addr()?;
    tokio::spawn(async move { axum::serve(listener, app).await });
    let stream = TcpStream::connect(address).await?;
    let (socket, _) = client_async(format!("ws://{address}/ws"), stream).await?;
    Ok(socket)
}

/// Reads until the server ends the session; `Ok(true)` means it closed within `limit`.
async fn closes_within(
    socket: &mut WebSocketStream<TcpStream>,
    limit: Duration,
) -> Result<bool, Box<dyn Error>> {
    let outcome = timeout(limit, async {
        loop {
            match socket.next().await {
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return,
                Some(Ok(_)) => {}
            }
        }
    })
    .await;
    Ok(outcome.is_ok())
}

#[tokio::test]
async fn responsive_client_keeps_receiving_samples_past_the_idle_timeout() -> TestResult {
    let mut socket = connect().await?;
    // Reading lets the client answer pings automatically, which counts as liveness.
    let mut samples = 0;
    let reading = timeout(FAST.idle_timeout * 3, async {
        while let Some(Ok(message)) = socket.next().await {
            if let Message::Binary(bytes) = message {
                assert_eq!(bytes.len(), 20);
                samples += 1;
            }
        }
    })
    .await;
    assert!(
        reading.is_err(),
        "session ended while the client was responsive"
    );
    assert!(samples >= 10, "received {samples} samples");
    Ok(())
}

#[tokio::test]
async fn silent_client_is_closed_after_the_idle_timeout() -> TestResult {
    let mut socket = connect().await?;
    // Not reading means pings go unanswered.
    tokio::time::sleep(FAST.idle_timeout * 2).await;
    assert!(closes_within(&mut socket, Duration::from_secs(2)).await?);
    Ok(())
}

#[tokio::test]
async fn oversized_inbound_message_closes_the_socket() -> TestResult {
    let mut socket = connect().await?;
    socket.send(Message::text("x".repeat(2 * 1024))).await?;
    // Well before the idle timeout could close it.
    assert!(closes_within(&mut socket, FAST.idle_timeout / 2).await?);
    Ok(())
}
