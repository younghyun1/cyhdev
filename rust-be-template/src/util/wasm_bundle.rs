use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::Path,
};

use anyhow::anyhow;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};

pub const HTML_CONTENT_TYPE: &str = "text/html; charset=utf-8";
pub const WASM_CONTENT_TYPE: &str = "application/wasm";

pub struct NormalizedBundle {
    pub gz_bytes: Vec<u8>,
    pub content_type: &'static str,
}

pub fn looks_like_html(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    let mut idx = 0;
    if data.len() >= 3 && data[0..3] == [0xef, 0xbb, 0xbf] {
        idx = 3;
    }

    while idx < data.len() && data[idx].is_ascii_whitespace() {
        idx += 1;
    }

    let head = &data[idx..];
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
    let mut out = Vec::new();
    let mut buf = [0u8; 8192];

    loop {
        let n = decoder.read(&mut buf)?;
        if n == 0 {
            break;
        }
        if out.len() + n > max_size {
            return Err(anyhow!("Decompressed bundle exceeds {max_size} bytes"));
        }
        out.extend_from_slice(&buf[..n]);
    }

    Ok(out)
}

/// Validates and gzip-normalizes a staged bundle through fixed-size buffers.
pub fn normalize_bundle_file(
    path: &Path,
    is_gzipped: bool,
    is_html: bool,
    max_decompressed_size: usize,
) -> anyhow::Result<NormalizedBundle> {
    let file = File::open(path)?;
    let buffered = BufReader::with_capacity(64 * 1024, file);
    if is_gzipped {
        normalize_bundle_reader(
            GzDecoder::new(buffered),
            is_html,
            max_decompressed_size,
        )
    } else {
        normalize_bundle_reader(buffered, is_html, max_decompressed_size)
    }
}

fn normalize_bundle_reader(
    mut reader: impl Read,
    is_html: bool,
    max_decompressed_size: usize,
) -> anyhow::Result<NormalizedBundle> {
    let mut encoder = GzEncoder::new(
        Vec::with_capacity(max_decompressed_size.min(4 * 1024 * 1024)),
        Compression::best(),
    );
    let mut buffer = [0_u8; 64 * 1024];
    let mut prefix = Vec::with_capacity(512);
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
        if prefix.len() < 512 {
            let prefix_bytes = (512 - prefix.len()).min(read);
            prefix.extend_from_slice(&buffer[..prefix_bytes]);
        }
        encoder.write_all(&buffer[..read])?;
    }

    if is_html {
        if !looks_like_html(&prefix) {
            return Err(anyhow!(
                "Bundle marked as HTML but contents do not look like HTML"
            ));
        }
    } else if !is_wasm_magic(&prefix) {
        return Err(anyhow!("Invalid WASM file (missing magic number)"));
    }

    let gz_bytes = encoder.finish()?;
    let max_encoded_size = max_decompressed_size.saturating_add(64 * 1024);
    if gz_bytes.len() > max_encoded_size {
        return Err(anyhow!(
            "Normalized bundle exceeds {max_encoded_size} bytes"
        ));
    }
    let content_type = if is_html {
        HTML_CONTENT_TYPE
    } else {
        WASM_CONTENT_TYPE
    };

    Ok(NormalizedBundle {
        gz_bytes,
        content_type,
    })
}

pub fn sniff_content_type_from_gzip_bytes(data: &[u8]) -> anyhow::Result<&'static str> {
    let decoder = GzDecoder::new(data);
    let mut head = Vec::with_capacity(512);
    decoder.take(512).read_to_end(&mut head)?;

    if is_wasm_magic(&head) {
        Ok(WASM_CONTENT_TYPE)
    } else if looks_like_html(&head) {
        Ok(HTML_CONTENT_TYPE)
    } else {
        Err(anyhow!("Unable to detect bundle content type"))
    }
}
