//! Bounded multipart staging for WebAssembly module assets.

use axum::extract::Multipart;
use tokio::io::AsyncReadExt;
use tracing::{error, info};

use crate::{
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    util::{
        media::{
            image_upload::is_allowed_image_mime,
            staged_upload::{
                StageUploadError, StagedUpload, read_bounded_text_field, stage_file_field,
            },
        },
        wasm_bundle::{is_wasm_magic, looks_like_html},
    },
};

pub(super) const MAX_BUNDLE_SIZE_BYTES: u64 = 50 * 1024 * 1024;
const MAX_THUMBNAIL_SIZE_BYTES: u64 = 10 * 1024 * 1024;
const MAX_TITLE_BYTES: usize = 512;
const MAX_DESCRIPTION_BYTES: usize = 64 * 1024;
const BUNDLE_PREFIX_BYTES: usize = 512;

pub(super) struct BundleUpload {
    pub(super) source: StagedUpload,
    pub(super) is_gzipped: bool,
    pub(super) is_html: bool,
}

pub(super) struct WasmAssetUpload {
    pub(super) bundle: Option<BundleUpload>,
    pub(super) thumbnail: Option<StagedUpload>,
    pub(super) title: Option<String>,
    pub(super) description: Option<String>,
}

impl WasmAssetUpload {
    pub(super) async fn read(multipart: &mut Multipart) -> HandlerResponse<Self> {
        let mut result = Self {
            bundle: None,
            thumbnail: None,
            title: None,
            description: None,
        };
        while let Some(field) = multipart.next_field().await.map_err(|source| {
            error!(error = %source, "Failed to read WASM asset multipart field");
            code_err(CodeError::FILE_UPLOAD_ERROR, source)
        })? {
            let name = field.name().map(str::to_owned);
            match name.as_deref() {
                Some("bundle_file") | Some("wasm_file") | Some("wasm") => {
                    if result.bundle.is_some() {
                        return Err(duplicate_field("bundle"));
                    }
                    let staged = stage_file_field(field, MAX_BUNDLE_SIZE_BYTES)
                        .await
                        .map_err(map_stage_error)?;
                    result.bundle = Some(classify_bundle(staged).await?);
                }
                Some("thumbnail") | Some("thumbnail_file") => {
                    if result.thumbnail.is_some() {
                        return Err(duplicate_field("thumbnail"));
                    }
                    if let Some(content_type) = field.content_type()
                        && !is_allowed_image_mime(content_type)
                    {
                        return Err(code_err(
                            CodeError::FILE_UPLOAD_ERROR,
                            "Unsupported thumbnail image type",
                        ));
                    }
                    result.thumbnail = Some(
                        stage_file_field(field, MAX_THUMBNAIL_SIZE_BYTES)
                            .await
                            .map_err(map_stage_error)?,
                    );
                }
                Some("title") | Some("wasm_module_title") => {
                    result.title = Some(
                        read_bounded_text_field(field, MAX_TITLE_BYTES)
                            .await
                            .map_err(map_stage_error)?,
                    );
                }
                Some("description") | Some("wasm_module_description") => {
                    result.description = Some(
                        read_bounded_text_field(field, MAX_DESCRIPTION_BYTES)
                            .await
                            .map_err(map_stage_error)?,
                    );
                }
                Some(other) => info!(field = other, "Ignored unknown WASM asset field"),
                None => info!("Ignored unnamed WASM asset field"),
            }
        }
        Ok(result)
    }
}

async fn classify_bundle(source: StagedUpload) -> HandlerResponse<BundleUpload> {
    let prefix = read_prefix(source.path()).await.map_err(|error| {
        error!(error = %error, "Failed to inspect staged WASM bundle");
        code_err(CodeError::FILE_UPLOAD_ERROR, error)
    })?;
    let file_name = source.file_name.as_deref().unwrap_or_default();
    let content_type = source.content_type.as_deref().unwrap_or_default();
    let is_gzipped = prefix.starts_with(&[0x1f, 0x8b])
        || file_name.ends_with(".gz")
        || content_type.contains("gzip");
    let declared_html = content_type.starts_with("text/html")
        || [".html", ".htm", ".html.gz", ".htm.gz"]
            .iter()
            .any(|suffix| file_name.ends_with(suffix));
    let declared_wasm = content_type.starts_with("application/wasm")
        || file_name.ends_with(".wasm")
        || file_name.ends_with(".wasm.gz");
    let is_html = if declared_html {
        true
    } else if declared_wasm || (!is_gzipped && is_wasm_magic(&prefix)) {
        false
    } else if !is_gzipped && looks_like_html(&prefix) {
        true
    } else if is_gzipped {
        return Err(code_err(
            CodeError::FILE_UPLOAD_ERROR,
            "Name compressed bundles .html.gz or .wasm.gz",
        ));
    } else {
        return Err(code_err(
            CodeError::FILE_UPLOAD_ERROR,
            "Unrecognized bundle type; expected HTML or WebAssembly",
        ));
    };
    Ok(BundleUpload {
        source,
        is_gzipped,
        is_html,
    })
}

async fn read_prefix(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    let file = tokio::fs::File::open(path).await?;
    let mut prefix = Vec::with_capacity(BUNDLE_PREFIX_BYTES);
    file.take(BUNDLE_PREFIX_BYTES as u64)
        .read_to_end(&mut prefix)
        .await?;
    Ok(prefix)
}

fn duplicate_field(name: &'static str) -> CodeErrorResp {
    code_err(
        CodeError::FILE_UPLOAD_ERROR,
        format!("Only one {name} field is allowed"),
    )
}

fn map_stage_error(error: StageUploadError) -> CodeErrorResp {
    code_err(CodeError::FILE_UPLOAD_ERROR, error)
}
