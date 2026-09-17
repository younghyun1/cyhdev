use std::error::Error;

use axum::{Router, http::StatusCode};
use reqwest::{Client, header};

/// Exercise the mounted router over HTTP, including the enclosing SPA fallback.
#[tokio::test]
async fn serves_files_and_revalidates_without_spa_fallback() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let web = directory.path().join("web");
    std::fs::create_dir(&web)?;
    std::fs::write(web.join("index.html"), "<title>Map</title>")?;
    std::fs::write(web.join("players.json"), "[]")?;
    std::fs::write(directory.path().join("secret.txt"), "private")?;
    let app = Router::new()
        .merge(super::router(Some(web)))
        .fallback(|| async { "website shell" });
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let origin = format!("http://{}", listener.local_addr()?);
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let redirect = client.get(format!("{origin}/minecraft/map")).send().await?;
    assert_eq!(redirect.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(redirect.headers()[header::LOCATION], "/minecraft/map/");
    let index = client
        .get(format!("{origin}/minecraft/map/"))
        .send()
        .await?;
    assert_eq!(index.status(), StatusCode::OK);
    assert_eq!(index.headers()[header::CACHE_CONTROL], "public, no-cache");
    let modified = index.headers()[header::LAST_MODIFIED].clone();
    assert_eq!(index.text().await?, "<title>Map</title>");
    let conditional = client
        .get(format!("{origin}/minecraft/map/index.html"))
        .header(header::IF_MODIFIED_SINCE, modified)
        .send()
        .await?;
    assert_eq!(conditional.status(), StatusCode::NOT_MODIFIED);
    let players = client
        .get(format!("{origin}/minecraft/map/players.json"))
        .send()
        .await?;
    assert_eq!(players.status(), StatusCode::OK);
    assert_eq!(players.headers()[header::CACHE_CONTROL], "no-store");
    for path in ["missing.png", "..%2fsecret.txt"] {
        let response = client
            .get(format!("{origin}/minecraft/map/{path}"))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert!(!response.text().await?.contains("website shell"));
    }
    let head = client
        .head(format!("{origin}/minecraft/map/index.html"))
        .send()
        .await?;
    assert_eq!(head.status(), StatusCode::OK);
    assert!(head.bytes().await?.is_empty());
    server.abort();
    Ok(())
}

#[tokio::test]
async fn unconfigured_map_is_explicitly_unavailable() -> Result<(), Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let url = format!("http://{}/minecraft/map/", listener.local_addr()?);
    let server = tokio::spawn(async move { axum::serve(listener, super::router(None)).await });
    let response = Client::new().get(url).send().await?;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    server.abort();
    Ok(())
}
