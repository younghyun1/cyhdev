//! Local OIDC code exchange with real signed ID tokens and PKCE validation.

use axum::{
    Form, Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use openidconnect::{
    PrivateSigningKey,
    core::{CoreJwsSigningAlgorithm, CoreRsaPrivateSigningKey},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;

pub struct Provider {
    pub key: CoreRsaPrivateSigningKey,
    pub codes: Mutex<BTreeMap<String, (String, String, i64)>>,
}

pub async fn discovery() -> Json<Value> {
    let base = format!("http://127.0.0.1:{}", crate::files::HTTP_PORT);
    Json(
        json!({"issuer":format!("{base}/"),"authorization_endpoint":format!("{base}/authorize"),"token_endpoint":format!("{base}/token"),
        "jwks_uri":format!("{base}/jwks"),"response_types_supported":["code"],"subject_types_supported":["public"],
        "id_token_signing_alg_values_supported":["RS256"],"token_endpoint_auth_methods_supported":["client_secret_basic","client_secret_post"],
        "code_challenge_methods_supported":["S256"],"scopes_supported":["openid","email","profile"]}),
    )
}

pub async fn jwks(State(provider): State<Arc<Provider>>) -> Json<Value> {
    Json(json!({"keys":[provider.key.as_verification_key()]}))
}

pub async fn authorize(
    State(provider): State<Arc<Provider>>,
    Query(query): Query<BTreeMap<String, String>>,
) -> Result<Response, StatusCode> {
    let get = |name: &str| {
        query
            .get(name)
            .filter(|value| value.len() <= 4096)
            .ok_or(StatusCode::BAD_REQUEST)
    };
    let redirect = get("redirect_uri")?;
    if redirect != "https://127.0.0.1:18443/api/auth/oidc/callback"
        || get("client_id")? != "optimization-fixture-client"
        || get("code_challenge_method")? != "S256"
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let code = uuid::Uuid::new_v4().to_string();
    let mut codes = provider.codes.lock().await;
    let now = chrono::Utc::now().timestamp();
    codes.retain(|_, (_, _, time)| now - *time < 60);
    if codes.len() >= 128 {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    codes.insert(
        code.clone(),
        (get("nonce")?.clone(), get("code_challenge")?.clone(), now),
    );
    let mut target = reqwest::Url::parse(redirect).map_err(|_| StatusCode::BAD_REQUEST)?;
    target
        .query_pairs_mut()
        .append_pair("code", &code)
        .append_pair("state", get("state")?);
    Ok(Redirect::to(target.as_str()).into_response())
}

pub async fn token(
    State(provider): State<Arc<Provider>>,
    headers: HeaderMap,
    Form(form): Form<BTreeMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let expected = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD
            .encode("optimization-fixture-client:optimization-fixture-oidc")
    );
    let authorized = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        == Some(expected.as_str())
        || (form.get("client_id").map(String::as_str) == Some("optimization-fixture-client")
            && form.get("client_secret").map(String::as_str) == Some("optimization-fixture-oidc"));
    if !authorized {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let code = form.get("code").ok_or(StatusCode::BAD_REQUEST)?;
    let verifier = form.get("code_verifier").ok_or(StatusCode::BAD_REQUEST)?;
    let (nonce, challenge, issued) = provider
        .codes
        .lock()
        .await
        .remove(code)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let now = chrono::Utc::now().timestamp();
    if now - issued > 60 || URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())) != challenge
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let header = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&json!({"alg":"RS256","kid":"optimization-fixture-key","typ":"JWT"}))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    let claims = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&json!({"iss":format!("http://127.0.0.1:{}/",crate::files::HTTP_PORT),
        "sub":"fixture-member","aud":"optimization-fixture-client","iat":now,"exp":now+60,"nonce":nonce,
        "email":"fixture-member@example.test","email_verified":true,"name":"fixture-member"})).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?);
    let input = format!("{header}.{claims}");
    let signature = provider
        .key
        .sign(
            &CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
            input.as_bytes(),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        json!({"access_token":"optimization-fixture-access","token_type":"Bearer","expires_in":60,"id_token":format!("{input}.{}",URL_SAFE_NO_PAD.encode(signature))}),
    ))
}
