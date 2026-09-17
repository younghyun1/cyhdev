use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri, header};
use rust_embed::Embed;

use super::{accepted_codings, serve_static};

#[derive(Embed)]
#[folder = "src/routers/main_router/static_asset_fixtures/"]
struct TestAssets;

fn request(path: &str, accept_encoding: &str) -> axum::response::Response {
    let uri = match path.parse::<Uri>() {
        Ok(uri) => uri,
        Err(error) => panic!("invalid test URI {path}: {error}"),
    };
    let mut headers = HeaderMap::new();
    let encoding = match HeaderValue::from_str(accept_encoding) {
        Ok(encoding) => encoding,
        Err(error) => panic!("invalid test encoding {accept_encoding}: {error}"),
    };
    headers.insert(header::ACCEPT_ENCODING, encoding);
    serve_static::<TestAssets>(&uri, &headers, &super::immutable_paths::<TestAssets>())
}

#[test]
fn accepted_encodings_follow_quality_then_server_preference() {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::ACCEPT_ENCODING,
        HeaderValue::from_static("gzip;q=0.7, zstd;q=0.9, identity;q=0.2"),
    );
    let codings = accepted_codings(&headers);

    assert_eq!(codings.len(), 3);
    assert_eq!(codings[0].coding, super::ContentCoding::Zstd);
    assert_eq!(codings[1].coding, super::ContentCoding::Gzip);
    assert_eq!(codings[2].coding, super::ContentCoding::Identity);
}

#[test]
fn zstd_acceptance_falls_back_to_available_gzip() {
    let response = request(
        "/eu5-locations-db/app/pkg/eu5_location_filter.js",
        "zstd, gzip;q=0.8",
    );

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_ENCODING),
        Some(&HeaderValue::from_static("gzip"))
    );
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE),
        Some(&HeaderValue::from_static("text/javascript"))
    );
    assert_eq!(
        response.headers().get(header::VARY),
        Some(&HeaderValue::from_static("Accept-Encoding"))
    );
}

#[test]
fn wasm_uses_its_uncompressed_path_for_mime_detection() {
    let response = request(
        "/eu5-locations-db/app/pkg/eu5_location_filter_bg.wasm",
        "gzip",
    );

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE),
        Some(&HeaderValue::from_static("application/wasm"))
    );
}

#[test]
fn etag_revalidation_returns_not_modified_with_cache_headers() {
    let response = request("/eu5-locations-db/app/index.html", "gzip");
    let etag = match response.headers().get(header::ETAG) {
        Some(etag) => etag.clone(),
        None => panic!("asset response omitted ETag"),
    };
    let etag_text = match etag.to_str() {
        Ok(etag) => etag,
        Err(error) => panic!("ETag was not text: {error}"),
    };
    assert!(etag_text.starts_with('"') && etag_text.ends_with('"'));
    assert_eq!(etag_text.len(), 66);

    let uri = Uri::from_static("/eu5-locations-db/app/index.html");
    let mut headers = HeaderMap::new();
    headers.insert(header::ACCEPT_ENCODING, HeaderValue::from_static("gzip"));
    headers.insert(header::IF_NONE_MATCH, etag);
    let response =
        serve_static::<TestAssets>(&uri, &headers, &super::immutable_paths::<TestAssets>());

    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL),
        Some(&HeaderValue::from_static(
            "public, max-age=0, must-revalidate"
        ))
    );
    assert_eq!(
        response.headers().get(header::VARY),
        Some(&HeaderValue::from_static("Accept-Encoding"))
    );
}

#[test]
fn missing_eu5_asset_never_uses_spa_fallback() {
    let response = request("/eu5-locations-db/app/missing.js", "gzip");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn refusing_gzip_rejects_gzip_only_eu5_assets() {
    let response = request("/eu5-locations-db/app/index.html", "gzip;q=0, identity;q=1");
    assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
}

#[test]
fn unknown_spa_path_uses_the_best_index_representation() {
    let response = request("/unknown/route", "zstd, gzip;q=0.8");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_ENCODING),
        Some(&HeaderValue::from_static("zstd"))
    );
}

#[test]
fn only_manifest_fingerprinted_assets_are_immutable() {
    for path in [
        "/assets/app-AbCd1234.js",
        "/assets/app-Xy_z-123.css",
        "/assets/icon-AbCd1234.svg",
    ] {
        let response = request(path, "identity");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], super::IMMUTABLE);
    }
    for path in [
        "/",
        "/minecraft",
        "/index.html",
        "/assets/settings.js",
        "/assets/public-AbCd1234.js",
        "/eu5-locations-db/app/index.html",
    ] {
        let response = request(path, "gzip, identity");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], super::REVALIDATE);
    }
}

#[test]
fn immutable_variants_keep_distinct_etags_and_conditional_headers()
-> Result<(), Box<dyn std::error::Error>> {
    let uri = Uri::from_static("/assets/app-AbCd1234.js?v=1");
    let immutable = super::immutable_paths::<TestAssets>();
    let mut etags = std::collections::HashSet::new();
    for encoding in ["identity", "gzip", "zstd"] {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT_ENCODING, HeaderValue::from_static(encoding));
        let response = serve_static::<TestAssets>(&uri, &headers, &immutable);
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], super::IMMUTABLE);
        assert_eq!(response.headers()[header::VARY], "Accept-Encoding");
        assert_eq!(response.headers()[header::CONTENT_TYPE], "text/javascript");
        if encoding == "identity" {
            assert!(!response.headers().contains_key(header::CONTENT_ENCODING));
        } else {
            assert_eq!(response.headers()[header::CONTENT_ENCODING], encoding);
        }
        let etag = response.headers()[header::ETAG].clone();
        assert!(etags.insert(etag.to_str()?.to_owned()));
        headers.insert(header::IF_NONE_MATCH, etag);
        let conditional = serve_static::<TestAssets>(&uri, &headers, &immutable);
        assert_eq!(conditional.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(
            conditional.headers()[header::CACHE_CONTROL],
            super::IMMUTABLE
        );
        assert_eq!(conditional.headers()[header::VARY], "Accept-Encoding");
    }
    Ok(())
}

#[test]
fn missing_or_unacceptable_assets_are_never_cached_or_served_as_html() {
    let missing = request("/assets/old-AbCd1234.js", "gzip");
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_eq!(missing.headers()[header::CACHE_CONTROL], "no-store");
    let unacceptable = request("/assets/app-AbCd1234.js", "*;q=0");
    assert_eq!(unacceptable.status(), StatusCode::NOT_ACCEPTABLE);
    assert_eq!(unacceptable.headers()[header::CACHE_CONTROL], "no-store");
}

#[test]
fn older_builds_without_a_manifest_revalidate_every_asset() {
    let uri = Uri::from_static("/assets/app-AbCd1234.js");
    let response = serve_static::<TestAssets>(&uri, &HeaderMap::new(), &Default::default());
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CACHE_CONTROL], super::REVALIDATE);
}
