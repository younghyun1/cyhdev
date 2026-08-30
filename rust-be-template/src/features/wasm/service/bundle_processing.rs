//! Bounded gzip normalization and WebAssembly/HTML validation.

use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::Path,
};

use anyhow::anyhow;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};

use super::super::domain::bundle::{NormalizedWasmBundle, WasmBundleKind};

pub const MAX_BUNDLE_SIZE_BYTES: u64 = 50 * 1024 * 1024;
const FILE_BUFFER_BYTES: usize = 64 * 1024;
const BUNDLE_PREFIX_BYTES: usize = 512;

pub fn looks_like_html(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }
    let mut index = if data.len() >= 3 && data[0..3] == [0xef, 0xbb, 0xbf] {
        3
    } else {
        0
    };
    while index < data.len() && data[index].is_ascii_whitespace() {
        index += 1;
    }
    let head = &data[index..];
    head.starts_with(b"<!DOCTYPE")
        || head.starts_with(b"<html")
        || head.starts_with(b"<HTML")
        || head.starts_with(b"<")
}

pub fn is_wasm_magic(data: &[u8]) -> bool {
    data.len() >= 4 && &data[0..4] == b"\x00asm"
}

pub fn gzip_decompress_limited(data: &[u8], max_size: usize) -> anyhow::Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut output = Vec::new();
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        let read = decoder.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        if output.len().saturating_add(read) > max_size {
            return Err(anyhow!("Decompressed bundle exceeds {max_size} bytes"));
        }
        output.extend_from_slice(&buffer[..read]);
    }
    Ok(output)
}

pub fn normalize_bundle_file(
    path: &Path,
    is_gzipped: bool,
    kind: WasmBundleKind,
    max_decompressed_size: usize,
) -> anyhow::Result<NormalizedWasmBundle> {
    let file = File::open(path)?;
    let buffered = BufReader::with_capacity(FILE_BUFFER_BYTES, file);
    if is_gzipped {
        normalize_reader(GzDecoder::new(buffered), kind, max_decompressed_size)
    } else {
        normalize_reader(buffered, kind, max_decompressed_size)
    }
}

fn normalize_reader(
    mut reader: impl Read,
    kind: WasmBundleKind,
    max_decompressed_size: usize,
) -> anyhow::Result<NormalizedWasmBundle> {
    let mut encoder = GzEncoder::new(
        Vec::with_capacity(max_decompressed_size.min(4 * 1024 * 1024)),
        Compression::best(),
    );
    let mut buffer = [0_u8; FILE_BUFFER_BYTES];
    let mut prefix = Vec::with_capacity(BUNDLE_PREFIX_BYTES);
    let mut total = 0_usize;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(read)
            .filter(|total| *total <= max_decompressed_size)
            .ok_or_else(|| anyhow!("Bundle exceeds {max_decompressed_size} bytes"))?;
        if prefix.len() < BUNDLE_PREFIX_BYTES {
            let count = (BUNDLE_PREFIX_BYTES - prefix.len()).min(read);
            prefix.extend_from_slice(&buffer[..count]);
        }
        encoder.write_all(&buffer[..read])?;
    }
    validate_prefix(&prefix, kind)?;
    let gz_bytes = encoder.finish()?;
    let max_encoded_size = max_decompressed_size.saturating_add(FILE_BUFFER_BYTES);
    if gz_bytes.len() > max_encoded_size {
        return Err(anyhow!("Normalized bundle exceeds {max_encoded_size} bytes"));
    }
    Ok(NormalizedWasmBundle { gz_bytes, kind })
}

fn validate_prefix(prefix: &[u8], kind: WasmBundleKind) -> anyhow::Result<()> {
    match kind {
        WasmBundleKind::Html if !looks_like_html(prefix) => {
            Err(anyhow!("Bundle marked as HTML but contents do not look like HTML"))
        }
        WasmBundleKind::WebAssembly if !is_wasm_magic(prefix) => {
            Err(anyhow!("Invalid WASM file (missing magic number)"))
        }
        WasmBundleKind::Html | WasmBundleKind::WebAssembly => Ok(()),
    }
}

pub fn sniff_kind_from_gzip_bytes(data: &[u8]) -> anyhow::Result<WasmBundleKind> {
    let decoder = GzDecoder::new(data);
    let mut head = Vec::with_capacity(BUNDLE_PREFIX_BYTES);
    decoder.take(BUNDLE_PREFIX_BYTES as u64).read_to_end(&mut head)?;
    if is_wasm_magic(&head) {
        Ok(WasmBundleKind::WebAssembly)
    } else if looks_like_html(&head) {
        Ok(WasmBundleKind::Html)
    } else {
        Err(anyhow!("Unable to detect bundle content type"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_signatures_are_classified_without_full_bundle_reads() {
        assert!(looks_like_html(b"\xef\xbb\xbf  <!DOCTYPE html>"));
        assert!(is_wasm_magic(b"\0asm\x01\0\0\0"));
        assert!(!looks_like_html(b"plain text"));
        assert!(!is_wasm_magic(b"not wasm"));
    }

    #[test]
    fn gzip_decompression_stops_at_the_declared_limit() -> anyhow::Result<()> {
        let source = vec![b'x'; 1_024];
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&source)?;
        let compressed = encoder.finish()?;
        assert!(gzip_decompress_limited(&compressed, 1_023).is_err());
        assert_eq!(gzip_decompress_limited(&compressed, 1_024)?, source);
        Ok(())
    }
}
