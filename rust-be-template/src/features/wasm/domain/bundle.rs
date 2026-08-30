//! Persistence-independent WebAssembly bundle values.

use std::sync::Arc;

pub const HTML_CONTENT_TYPE: &str = "text/html; charset=utf-8";
pub const WASM_CONTENT_TYPE: &str = "application/wasm";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasmBundleKind {
    Html,
    WebAssembly,
}

impl WasmBundleKind {
    pub fn content_type(self) -> &'static str {
        match self {
            Self::Html => HTML_CONTENT_TYPE,
            Self::WebAssembly => WASM_CONTENT_TYPE,
        }
    }
}

pub struct NormalizedWasmBundle {
    pub gz_bytes: Vec<u8>,
    pub kind: WasmBundleKind,
}

#[derive(Clone)]
pub struct CachedWasmBundle {
    pub bytes: Arc<[u8]>,
    pub is_gzipped: bool,
    pub kind: WasmBundleKind,
}

pub struct ServedWasmBundle {
    pub bytes: Arc<[u8]>,
    pub content_type: &'static str,
    pub content_encoding_gzip: bool,
}
