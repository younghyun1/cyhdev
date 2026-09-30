//! Authenticated, bounded acknowledgements for the dedicated Minecraft fixture.

use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::Message},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::get,
};
use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::Semaphore;

struct Management {
    runtime: PathBuf,
    admission: Arc<Semaphore>,
}

pub async fn serve(runtime: PathBuf) -> anyhow::Result<()> {
    let state = Arc::new(Management {
        runtime,
        admission: Arc::new(Semaphore::new(4)),
    });
    let router = Router::new().route("/", get(upgrade)).with_state(state);
    let listener = tokio::net::TcpListener::bind((
        std::net::Ipv4Addr::LOCALHOST,
        crate::files::MANAGEMENT_PORT,
    ))
    .await?;
    axum::serve(listener, router).await?;
    Ok(())
}

async fn upgrade(
    State(state): State<Arc<Management>>,
    headers: HeaderMap,
    socket: WebSocketUpgrade,
) -> Result<Response, StatusCode> {
    if headers.get("authorization").and_then(|h| h.to_str().ok())
        != Some("Bearer optimizationfixture012345678901234567890")
    {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let permit = state
        .admission
        .clone()
        .try_acquire_owned()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(socket
        .max_message_size(16384)
        .on_upgrade(move |mut socket| async move {
            for _ in 0..64 {
                let text = match tokio::time::timeout(Duration::from_secs(30), socket.recv()).await
                {
                    Ok(Some(Ok(Message::Text(text)))) => text,
                    _ => break,
                };
                let value: Value = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(_) => break,
                };
                let method = value["method"].as_str().unwrap_or("");
                let result = match method {
                    "minecraft:players" | "minecraft:allowlist" => {
                        json!([{"id":crate::files::PLAYER,"name":"FixturePlayer"}])
                    }
                    "minecraft:serversettings/use_allowlist" => json!(true),
                    "minecraft:serversettings/use_allowlist/set" => value["params"][0].clone(),
                    "minecraft:allowlist/add"
                    | "minecraft:allowlist/remove"
                    | "minecraft:players/kick" => json!([]),
                    "minecraft:server/system_message"
                    | "minecraft:server/save"
                    | "minecraft:server/stop" => json!(true),
                    _ => break,
                };
                let bytes =
                    match serde_json::to_vec(&json!({"method":method,"params":value["params"]})) {
                        Ok(bytes) => bytes,
                        Err(_) => break,
                    };
                if std::fs::write(state.runtime.join("management-last.json"), bytes).is_err() {
                    break;
                }
                let response = json!({"jsonrpc":"2.0","id":value["id"],"result":result});
                if socket
                    .send(Message::Text(response.to_string().into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            drop(permit);
        }))
}
