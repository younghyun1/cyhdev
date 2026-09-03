use axum::{
    http::{HeaderMap, HeaderValue, StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use mime_guess::from_path;
use rust_embed::{Embed, EmbeddedFile};

const CACHE_POLICY: &str = "public, max-age=0, must-revalidate";
const EU5_APPLICATION_PREFIX: &str = "eu5-locations-db/app/";

#[derive(Embed)]
#[folder = "../solid-csr-spa-template/dist/"]
struct EmbeddedAssets;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContentCoding {
    Zstd,
    Gzip,
    Identity,
}

impl ContentCoding {
    const fn priority(self) -> u8 {
        match self {
            Self::Zstd => 3,
            Self::Gzip => 2,
            Self::Identity => 1,
        }
    }

    const fn extension(self) -> &'static str {
        match self {
            Self::Zstd => ".zst",
            Self::Gzip => ".gz",
            Self::Identity => "",
        }
    }

    const fn header_value(self) -> Option<&'static str> {
        match self {
            Self::Zstd => Some("zstd"),
            Self::Gzip => Some("gzip"),
            Self::Identity => None,
        }
    }
}

#[derive(Clone, Copy)]
struct Preference {
    coding: ContentCoding,
    quality: f32,
}

fn parse_quality(raw: &str) -> f32 {
    match raw.trim().parse::<f32>() {
        Ok(value) if (0.0..=1.0).contains(&value) => value,
        Ok(_) | Err(_) => 0.0,
    }
}

fn set_max_quality(slot: &mut Option<f32>, quality: f32) {
    if slot.is_none_or(|current| current < quality) {
        *slot = Some(quality);
    }
}

fn accepted_codings(headers: &HeaderMap) -> Vec<Preference> {
    let mut zstd = None;
    let mut gzip = None;
    let mut identity = None;
    let mut wildcard = None;
    let mut header_present = false;

    for value in headers.get_all(header::ACCEPT_ENCODING) {
        header_present = true;
        let Ok(value) = value.to_str() else { continue };
        for entry in value.split(',') {
            let mut parts = entry.trim().split(';');
            let Some(name) = parts.next() else { continue };
            let mut quality = 1.0;
            for parameter in parts {
                let Some((key, value)) = parameter.trim().split_once('=') else {
                    continue;
                };
                if key.trim().eq_ignore_ascii_case("q") {
                    quality = parse_quality(value);
                }
            }
            match name.trim().to_ascii_lowercase().as_str() {
                "zstd" => set_max_quality(&mut zstd, quality),
                "gzip" | "x-gzip" => set_max_quality(&mut gzip, quality),
                "identity" => set_max_quality(&mut identity, quality),
                "*" => set_max_quality(&mut wildcard, quality),
                _ => {}
            }
        }
    }

    if !header_present {
        return vec![Preference {
            coding: ContentCoding::Identity,
            quality: 1.0,
        }];
    }

    let wildcard_quality = wildcard.unwrap_or(0.0);
    let identity_quality =
        identity.unwrap_or_else(|| if wildcard == Some(0.0) { 0.0 } else { 1.0 });
    let mut preferences = Vec::with_capacity(3);
    for preference in [
        Preference {
            coding: ContentCoding::Zstd,
            quality: zstd.unwrap_or(wildcard_quality),
        },
        Preference {
            coding: ContentCoding::Gzip,
            quality: gzip.unwrap_or(wildcard_quality),
        },
        Preference {
            coding: ContentCoding::Identity,
            quality: identity_quality,
        },
    ] {
        if preference.quality > 0.0 {
            preferences.push(preference);
        }
    }
    preferences.sort_by(|left, right| {
        right
            .quality
            .total_cmp(&left.quality)
            .then_with(|| right.coding.priority().cmp(&left.coding.priority()))
    });
    preferences
}

enum AssetOutcome {
    Found(Response),
    NotAcceptable,
    NotFound,
}

fn serve_path<A: Embed>(
    path: &str,
    preferences: &[Preference],
    request_headers: &HeaderMap,
) -> AssetOutcome {
    for preference in preferences {
        let representation_path = format!("{path}{}", preference.coding.extension());
        if let Some(content) = A::get(&representation_path) {
            return AssetOutcome::Found(asset_response(
                path,
                preference.coding,
                content,
                request_headers,
            ));
        }
    }

    let has_representation = [
        ContentCoding::Zstd,
        ContentCoding::Gzip,
        ContentCoding::Identity,
    ]
    .into_iter()
    .any(|coding| A::get(&format!("{path}{}", coding.extension())).is_some());
    if has_representation {
        AssetOutcome::NotAcceptable
    } else {
        AssetOutcome::NotFound
    }
}

fn strong_etag(content: &EmbeddedFile) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut etag = String::with_capacity(66);
    etag.push('"');
    for byte in content.metadata.sha256_hash() {
        etag.push(char::from(HEX[usize::from(byte >> 4)]));
        etag.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    etag.push('"');
    etag
}

fn if_none_match(headers: &HeaderMap, etag: &str) -> bool {
    headers.get_all(header::IF_NONE_MATCH).iter().any(|value| {
        value.to_str().is_ok_and(|value| {
            value.split(',').any(|candidate| {
                let candidate = candidate.trim();
                candidate == "*" || candidate == etag || candidate.strip_prefix("W/") == Some(etag)
            })
        })
    })
}

fn asset_response(
    path: &str,
    coding: ContentCoding,
    content: EmbeddedFile,
    request_headers: &HeaderMap,
) -> Response {
    let etag = strong_etag(&content);
    let Ok(etag_header) = HeaderValue::from_str(&etag) else {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid embedded asset hash",
        );
    };
    let not_modified = if_none_match(request_headers, &etag);
    let mut response = if not_modified {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        (StatusCode::OK, content.data).into_response()
    };
    let response_headers = response.headers_mut();
    response_headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
    response_headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(CACHE_POLICY),
    );
    response_headers.insert(header::ETAG, etag_header);
    if let Some(encoding) = coding.header_value() {
        response_headers.insert(header::CONTENT_ENCODING, HeaderValue::from_static(encoding));
    }
    if !not_modified {
        let mime = from_path(path).first_or_octet_stream();
        let content_type = match HeaderValue::from_str(mime.as_ref()) {
            Ok(content_type) => content_type,
            Err(_) => HeaderValue::from_static("application/octet-stream"),
        };
        response_headers.insert(header::CONTENT_TYPE, content_type);
    }
    response
}

fn error_response(status: StatusCode, message: &'static str) -> Response {
    let mut response = (status, message).into_response();
    response
        .headers_mut()
        .insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(CACHE_POLICY),
    );
    response
}

fn serve_static<A: Embed>(uri: &Uri, headers: &HeaderMap) -> Response {
    let path = match uri.path().trim_start_matches('/') {
        "" => "index.html",
        path => path,
    };
    let preferences = accepted_codings(headers);
    match serve_path::<A>(path, &preferences, headers) {
        AssetOutcome::Found(response) => response,
        AssetOutcome::NotAcceptable => error_response(
            StatusCode::NOT_ACCEPTABLE,
            "No acceptable asset representation",
        ),
        AssetOutcome::NotFound
            if path == "eu5-locations-db/app" || path.starts_with(EU5_APPLICATION_PREFIX) =>
        {
            error_response(StatusCode::NOT_FOUND, "EU5 application asset not found")
        }
        AssetOutcome::NotFound => match serve_path::<A>("index.html", &preferences, headers) {
            AssetOutcome::Found(response) => response,
            AssetOutcome::NotAcceptable => error_response(
                StatusCode::NOT_ACCEPTABLE,
                "No acceptable asset representation",
            ),
            AssetOutcome::NotFound => error_response(StatusCode::NOT_FOUND, "Not Found"),
        },
    }
}

/// Serves embedded static files using the best available accepted representation.
pub(super) async fn static_asset_handler(uri: Uri, headers: HeaderMap) -> impl IntoResponse {
    serve_static::<EmbeddedAssets>(&uri, &headers)
}

#[cfg(test)]
#[path = "static_assets_tests.rs"]
mod tests;
