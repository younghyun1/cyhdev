//! Bounded local object storage, mailbox inspection and fixture resets.

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{any, get, post},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    files,
    provider::{self, Provider},
};

struct Web {
    runtime: PathBuf,
    objects: Mutex<BTreeMap<String, Bytes>>,
    provider: Arc<Provider>,
}

pub async fn serve(runtime: PathBuf) -> anyhow::Result<()> {
    let key = openidconnect::core::CoreRsaPrivateSigningKey::from_pem(
        &std::fs::read_to_string(runtime.join("oidc-key.pem"))?,
        Some(openidconnect::JsonWebKeyId::new(
            "optimization-fixture-key".into(),
        )),
    )
    .map_err(anyhow::Error::msg)?;
    let provider = Arc::new(Provider {
        key,
        codes: Mutex::new(BTreeMap::new()),
    });
    let state = Arc::new(Web {
        runtime,
        provider: provider.clone(),
        objects: Mutex::new(BTreeMap::new()),
    });
    let oidc = Router::new()
        .route(
            "/.well-known/openid-configuration",
            get(provider::discovery),
        )
        .route("/jwks", get(provider::jwks))
        .route("/authorize", get(provider::authorize))
        .route("/token", post(provider::token))
        .with_state(provider);
    let router = Router::new()
        .route("/__fixture/reset", post(reset))
        .route("/__fixture/mail", get(mail))
        .route("/__fixture/visibility", post(visibility))
        .route("/__fixture/image", get(image))
        .route("/__fixture/health", get(|| async { "ready" }))
        .route("/{bucket}/{*key}", any(object))
        .layer(DefaultBodyLimit::max(16 * 1024 * 1024))
        .with_state(state)
        .merge(oidc);
    let listener =
        tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, files::HTTP_PORT)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

async fn visibility(State(state): State<Arc<Web>>, body: String) -> Result<StatusCode, StatusCode> {
    if body != "true" && body != "false" {
        return Err(StatusCode::BAD_REQUEST);
    }
    std::fs::write(state.runtime.join("visibility.json"), body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn reset(State(state): State<Arc<Web>>) -> Result<Json<Value>, StatusCode> {
    state.objects.lock().await.clear();
    state.provider.codes.lock().await.clear();
    for entry in std::fs::read_dir(state.runtime.join("mail"))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        let entry = entry.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        std::fs::remove_file(entry.path()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    std::fs::write(state.runtime.join("visibility.json"), "true")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    std::fs::write(state.runtime.join("player-hidden.json"), "0")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({"success":true})))
}

async fn image(State(state): State<Arc<Web>>) -> Result<Response, StatusCode> {
    let bytes = std::fs::read(state.runtime.join("image.png"))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes).into_response())
}

async fn mail(
    State(state): State<Arc<Web>>,
    Query(query): Query<BTreeMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let recipient = query.get("recipient").ok_or(StatusCode::BAD_REQUEST)?;
    if !recipient.ends_with("@example.test") {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut matches = Vec::new();
    for entry in std::fs::read_dir(state.runtime.join("mail"))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        let entry = entry.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let value: Value = serde_json::from_slice(
            &std::fs::read(entry.path()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if value["recipients"]
            .as_array()
            .is_some_and(|items| items.contains(&json!(recipient)))
        {
            matches.push((
                entry
                    .metadata()
                    .and_then(|meta| meta.modified())
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                value,
            ));
        }
    }
    matches.sort_by_key(|(time, _)| *time);
    match matches.pop() {
        Some((_, value)) => Ok(Json(value)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn object(
    State(state): State<Arc<Web>>,
    Path((bucket, key)): Path<(String, String)>,
    method: axum::http::Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, StatusCode> {
    if bucket != "cyhdev-img"
        || key.len() > 512
        || !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'.' | b'-' | b'_'))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut objects = state.objects.lock().await;
    match method {
        axum::http::Method::PUT | axum::http::Method::DELETE => {
            if !headers
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|value| value.contains("Credential=optimization-fixture-"))
            {
                return Err(StatusCode::UNAUTHORIZED);
            }
            if method == axum::http::Method::DELETE {
                objects.remove(&key);
                return Ok(StatusCode::NO_CONTENT.into_response());
            }
            let bytes: usize = objects.values().map(Bytes::len).sum();
            if objects.len() >= 256 || bytes.saturating_add(body.len()) > 128 * 1024 * 1024 {
                return Err(StatusCode::INSUFFICIENT_STORAGE);
            }
            let etag = format!("\"{}\"", files::digest(&body));
            objects.insert(key, body);
            Ok(([(header::ETAG, etag)], "").into_response())
        }
        axum::http::Method::GET | axum::http::Method::HEAD => {
            let body = objects.get(&key).ok_or(StatusCode::NOT_FOUND)?;
            Ok(([(header::CONTENT_TYPE, "image/avif")], body.clone()).into_response())
        }
        _ => Err(StatusCode::METHOD_NOT_ALLOWED),
    }
}
