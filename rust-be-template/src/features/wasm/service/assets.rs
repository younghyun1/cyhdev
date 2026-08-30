//! Bounded bundle classification and thumbnail preparation.

use tokio::io::AsyncReadExt;
use uuid::Uuid;

use crate::util::{
    image::{
        image_variant::{CyhdevImageType, IMAGE_ENCODING_FORMAT},
        map_image_format_to_db_enum::map_image_format_to_str,
        process_uploaded_image_files::{ProcessedImageFile, process_uploaded_image_files},
    },
    media::{
        object_store::ObjectLocation,
        persistence::PendingMediaObject,
        staged_upload::StagedUpload,
    },
    s3::AWS_S3_BUCKET_NAME,
};

use super::{
    asset_inputs::StagedBundleUpload,
    bundle_processing::{
        MAX_BUNDLE_SIZE_BYTES, is_wasm_magic, looks_like_html, normalize_bundle_file,
    },
};
use super::super::{
    domain::bundle::{NormalizedWasmBundle, WasmBundleKind},
    error::WasmError,
};

const BUNDLE_PREFIX_BYTES: usize = 512;

pub(super) struct PreparedThumbnail {
    pub url: String,
    pub pending: PendingMediaObject,
    _processed: ProcessedImageFile,
}

pub(super) async fn prepare_bundle(
    bundle: StagedBundleUpload,
) -> Result<NormalizedWasmBundle, WasmError> {
    let (is_gzipped, kind) = classify_bundle(&bundle.source).await?;
    let path = bundle.source.path().to_path_buf();
    let normalized = tokio::task::spawn_blocking(move || {
        normalize_bundle_file(
            &path,
            is_gzipped,
            kind,
            MAX_BUNDLE_SIZE_BYTES as usize,
        )
    })
    .await?
    .map_err(WasmError::Bundle)?;
    drop(bundle.source);
    Ok(normalized)
}

pub(super) async fn prepare_thumbnail(
    module_id: Uuid,
    thumbnail: StagedUpload,
    region: &str,
) -> Result<PreparedThumbnail, WasmError> {
    let mut outputs = process_uploaded_image_files(
        thumbnail.path(),
        None,
        vec![CyhdevImageType::DemoThumbnail],
    )
    .await
    .map_err(WasmError::Image)?
    .into_iter();
    let processed = match outputs.next() {
        Some(processed) => processed,
        None => {
            return Err(WasmError::Image(anyhow::anyhow!(
                "Thumbnail encoder produced no output"
            )));
        }
    };
    drop(thumbnail);
    let asset_id = Uuid::now_v7();
    let (extension, _) = map_image_format_to_str(IMAGE_ENCODING_FORMAT);
    let location = ObjectLocation::new(
        AWS_S3_BUCKET_NAME,
        format!("wasm-thumbnails/{module_id}/{asset_id}.{extension}"),
    );
    let url = location.public_s3_url(region);
    let pending = PendingMediaObject {
        location,
        content_type: "image/avif".to_owned(),
        source: processed.path_buf(),
    };
    Ok(PreparedThumbnail {
        url,
        pending,
        _processed: processed,
    })
}

async fn classify_bundle(source: &StagedUpload) -> Result<(bool, WasmBundleKind), WasmError> {
    let prefix = read_prefix(source.path())
        .await
        .map_err(|error| WasmError::Bundle(anyhow::Error::from(error)))?;
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
    let kind = if declared_html {
        WasmBundleKind::Html
    } else if declared_wasm || (!is_gzipped && is_wasm_magic(&prefix)) {
        WasmBundleKind::WebAssembly
    } else if !is_gzipped && looks_like_html(&prefix) {
        WasmBundleKind::Html
    } else if is_gzipped {
        return Err(WasmError::Bundle(anyhow::anyhow!(
            "Name compressed bundles .html.gz or .wasm.gz"
        )));
    } else {
        return Err(WasmError::Bundle(anyhow::anyhow!(
            "Unrecognized bundle type; expected HTML or WebAssembly"
        )));
    };
    Ok((is_gzipped, kind))
}

async fn read_prefix(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    let file = tokio::fs::File::open(path).await?;
    let mut prefix = Vec::with_capacity(BUNDLE_PREFIX_BYTES);
    file.take(BUNDLE_PREFIX_BYTES as u64)
        .read_to_end(&mut prefix)
        .await?;
    Ok(prefix)
}
