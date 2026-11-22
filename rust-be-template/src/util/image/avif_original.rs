//! Signature check for AVIF photographs that are stored exactly as uploaded.
//!
//! No AV1 decoder is compiled into the server, so an AVIF original is never
//! decoded here. The check only confirms that the file opens with an ISO base
//! media `ftyp` box naming the `avif` still-image brand, which keeps other
//! content from being published under an `image/avif` content type.

use std::{fs::File, io::Read, path::Path};

use anyhow::anyhow;

/// Bytes read from the start of the file. A `ftyp` box holds a major brand, a
/// minor version, and a short list of compatible brands, so real files fit in
/// far less; a longer box is refused instead of being read further.
const FTYP_PROBE_BYTES: usize = 512;

/// Size of the box header, major brand, and minor version before the
/// compatible-brand list.
const FTYP_FIXED_BYTES: usize = 16;

const STILL_IMAGE_BRAND: &[u8; 4] = b"avif";

/// Rejects a file that does not start with an AVIF still-image `ftyp` box.
///
/// Reads at most [`FTYP_PROBE_BYTES`] from the file; this blocks, so call it
/// from a blocking context.
pub fn require_avif_signature(source: &Path) -> anyhow::Result<()> {
    let mut head = Vec::with_capacity(FTYP_PROBE_BYTES);
    File::open(source)?
        .take(FTYP_PROBE_BYTES as u64)
        .read_to_end(&mut head)?;
    if has_avif_still_image_brand(&head) {
        Ok(())
    } else {
        Err(anyhow!(
            "Image content is not an AVIF still image despite its declared type"
        ))
    }
}

/// Returns whether `head` begins with a complete `ftyp` box whose major or
/// compatible brands include `avif`.
///
/// An image sequence that only declares `avis` is refused: it is an animation
/// container, not a photograph.
fn has_avif_still_image_brand(head: &[u8]) -> bool {
    let Some(size_bytes) = head.first_chunk::<4>() else {
        return false;
    };
    // Sizes 0 (box runs to end of file) and 1 (64-bit size follows) are legal
    // for other boxes but never produced for `ftyp`, and both fall below the
    // fixed part, so they are refused here.
    let Ok(box_size) = usize::try_from(u32::from_be_bytes(*size_bytes)) else {
        return false;
    };
    if box_size < FTYP_FIXED_BYTES || box_size > head.len() || !box_size.is_multiple_of(4) {
        return false;
    }
    if head.get(4..8) != Some(b"ftyp".as_slice()) {
        return false;
    }
    let Some(brands) = head.get(8..box_size) else {
        return false;
    };
    // The minor version sits between the major brand and the compatible
    // brands; it is a number, so it must not be matched as a brand.
    brands
        .chunks_exact(4)
        .enumerate()
        .any(|(index, brand)| index != 1 && brand == STILL_IMAGE_BRAND)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use image::{ImageFormat, Rgba, RgbaImage};

    use super::{has_avif_still_image_brand, require_avif_signature};

    fn ftyp(major: &[u8; 4], minor: [u8; 4], compatible: &[&[u8; 4]]) -> Vec<u8> {
        let size = 16 + 4 * compatible.len();
        let mut bytes = (size as u32).to_be_bytes().to_vec();
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(major);
        bytes.extend_from_slice(&minor);
        for brand in compatible {
            bytes.extend_from_slice(*brand);
        }
        bytes
    }

    #[test]
    fn accepts_avif_as_major_or_compatible_brand() {
        assert!(has_avif_still_image_brand(&ftyp(
            b"avif",
            [0; 4],
            &[b"mif1", b"miaf"]
        )));
        assert!(has_avif_still_image_brand(&ftyp(
            b"mif1",
            [0; 4],
            &[b"avif", b"miaf"]
        )));
        assert!(has_avif_still_image_brand(&ftyp(b"avif", [0; 4], &[])));
    }

    #[test]
    fn refuses_other_containers_sequences_and_malformed_boxes() {
        assert!(!has_avif_still_image_brand(&ftyp(
            b"heic",
            [0; 4],
            &[b"mif1"]
        )));
        assert!(!has_avif_still_image_brand(&ftyp(
            b"avis",
            [0; 4],
            &[b"msf1"]
        )));
        // `avif` in the minor-version slot is a number, not a brand.
        assert!(!has_avif_still_image_brand(&ftyp(
            b"isom",
            *b"avif",
            &[b"mp41"]
        )));
        assert!(!has_avif_still_image_brand(b""));
        assert!(!has_avif_still_image_brand(
            b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"
        ));
        // Declared size runs past the bytes that were read.
        let mut truncated = ftyp(b"avif", [0; 4], &[b"mif1"]);
        truncated.truncate(18);
        assert!(!has_avif_still_image_brand(&truncated));
        // Box type is not `ftyp`.
        let mut wrong_type = ftyp(b"avif", [0; 4], &[]);
        wrong_type[4..8].copy_from_slice(b"moov");
        assert!(!has_avif_still_image_brand(&wrong_type));
        // Sizes 0 and 1 are below the fixed part of the box.
        for size in [0_u32, 1, 15] {
            let mut bytes = ftyp(b"avif", [0; 4], &[]);
            bytes[..4].copy_from_slice(&size.to_be_bytes());
            assert!(!has_avif_still_image_brand(&bytes), "size {size}");
        }
    }

    #[test]
    fn encoder_output_passes_and_a_mislabelled_png_is_refused() -> anyhow::Result<()> {
        let avif = tempfile::Builder::new().suffix(".avif").tempfile()?;
        RgbaImage::from_pixel(16, 16, Rgba([10, 120, 200, 255]))
            .save_with_format(avif.path(), ImageFormat::Avif)?;
        assert!(require_avif_signature(avif.path()).is_ok());

        let mut png = tempfile::NamedTempFile::new()?;
        png.write_all(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR")?;
        assert!(require_avif_signature(png.path()).is_err());
        Ok(())
    }
}
