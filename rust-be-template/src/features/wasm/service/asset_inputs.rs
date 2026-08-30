//! Staged upload inputs handed from the HTTP parser to WebAssembly use cases.

use crate::util::media::staged_upload::StagedUpload;

pub struct StagedBundleUpload {
    pub source: StagedUpload,
}

pub struct StagedWasmAssets {
    pub bundle: Option<StagedBundleUpload>,
    pub thumbnail: Option<StagedUpload>,
    pub title: Option<String>,
    pub description: Option<String>,
}
