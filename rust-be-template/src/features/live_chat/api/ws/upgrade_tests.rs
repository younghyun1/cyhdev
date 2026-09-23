//! Socket-level coverage for the upgrade limits: frames at the protocol
//! maximum are delivered, larger ones fail inside the codec.

use std::net::SocketAddr;

use axum::{
    Router,
    extract::ws::{Message, WebSocketUpgrade},
    response::Response,
    routing::get,
};
use futures_util::SinkExt;
use tokio::{net::TcpListener, sync::mpsc};
use tokio_tungstenite::tungstenite::Message as ClientMessage;

use super::{LIVE_CHAT_MAX_FRAME_BYTES, upgrade::apply_connection_limits};

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// What the server-side reader observed for each inbound frame.
#[derive(Debug, PartialEq)]
enum Observed {
    Binary(usize),
    Error,
    End,
}

async fn start_server() -> Result<(SocketAddr, mpsc::Receiver<Observed>), std::io::Error> {
    let (observed_tx, observed_rx) = mpsc::channel(8);
    let router = Router::new().route(
        "/ws",
        get(move |ws: WebSocketUpgrade| {
            let observed_tx = observed_tx.clone();
            async move {
                let response: Response =
                    apply_connection_limits(ws).on_upgrade(|mut socket| async move {
                        loop {
                            let observed = match socket.recv().await {
                                Some(Ok(Message::Binary(bytes))) => Observed::Binary(bytes.len()),
                                Some(Ok(_)) => continue,
                                Some(Err(_)) => Observed::Error,
                                None => Observed::End,
                            };
                            let finished = observed != Observed::Binary(LIVE_CHAT_MAX_FRAME_BYTES);
                            if observed_tx.send(observed).await.is_err() || finished {
                                return;
                            }
                        }
                    });
                response
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    Ok((address, observed_rx))
}

#[tokio::test]
async fn frames_above_the_protocol_limit_fail_before_delivery() -> TestResult {
    let (address, mut observed) = start_server().await?;
    let stream = tokio::net::TcpStream::connect(address).await?;
    let url = format!("ws://{address}/ws");
    let (mut client, _) = tokio_tungstenite::client_async(url.as_str(), stream).await?;

    client
        .send(ClientMessage::Binary(
            vec![7_u8; LIVE_CHAT_MAX_FRAME_BYTES].into(),
        ))
        .await?;
    assert_eq!(
        observed.recv().await,
        Some(Observed::Binary(LIVE_CHAT_MAX_FRAME_BYTES))
    );

    // A frame one byte over the limit must never reach the handler; the
    // codec rejects it and the server stops reading this connection.
    let _ = client
        .send(ClientMessage::Binary(
            vec![7_u8; LIVE_CHAT_MAX_FRAME_BYTES + 1].into(),
        ))
        .await;
    assert_eq!(observed.recv().await, Some(Observed::Error));
    Ok(())
}
