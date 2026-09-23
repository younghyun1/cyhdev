//! Image-upload metadata validation shared by media endpoints.
//!
//! Each accepted MIME type maps to exactly one decoder. Uploads are decoded
//! with that declared format rather than whatever the bytes sniff as, so only
//! the codecs compiled into `image` (see `rust-be-template/Cargo.toml`) are
//! reachable, and a file whose signature disagrees with its declared type is
//! rejected before decoding.

use image::ImageFormat;

/// MIME types accepted for image uploads, each with the decoder it selects.
pub const ALLOWED_IMAGE_FORMATS: [(&str, ImageFormat); 6] = [
    ("image/png", ImageFormat::Png),
    ("image/jpeg", ImageFormat::Jpeg),
    ("image/gif", ImageFormat::Gif),
    ("image/webp", ImageFormat::WebP),
    ("image/tiff", ImageFormat::Tiff),
    ("image/bmp", ImageFormat::Bmp),
];

/// Decoder for a declared MIME type, or `None` when the type is not accepted.
pub fn image_format_for_mime(content_type: &str) -> Option<ImageFormat> {
    let media_type = content_type
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or_default();
    ALLOWED_IMAGE_FORMATS
        .iter()
        .find(|(mime, _)| mime.eq_ignore_ascii_case(media_type))
        .map(|(_, format)| *format)
}

/// Returns whether the declared MIME type is supported for image uploads.
pub fn is_allowed_image_mime(content_type: &str) -> bool {
    image_format_for_mime(content_type).is_some()
}

/// Resolves a staged upload's declared type to its decoder.
pub fn declared_image_format(content_type: Option<&str>) -> anyhow::Result<ImageFormat> {
    content_type
        .and_then(image_format_for_mime)
        .ok_or_else(|| anyhow::anyhow!("Image upload declared no supported image type"))
}

/// Returns whether a client filename carries a non-empty extension.
pub fn has_file_extension(file_name: &str) -> bool {
    match file_name.rsplit_once('.') {
        Some((_, extension)) => !extension.is_empty(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use image::ImageFormat;

    use super::{
        declared_image_format, has_file_extension, image_format_for_mime, is_allowed_image_mime,
    };

    #[test]
    fn validates_declared_image_metadata() {
        assert!(is_allowed_image_mime("image/jpeg"));
        assert!(!is_allowed_image_mime("application/pdf"));
        assert!(has_file_extension("photo.jpeg"));
        assert!(!has_file_extension("photo"));
        assert!(!has_file_extension("photo."));
    }

    #[test]
    fn declared_types_map_to_one_decoder_and_exotic_codecs_are_refused() {
        assert_eq!(image_format_for_mime("image/png"), Some(ImageFormat::Png));
        assert_eq!(
            image_format_for_mime("IMAGE/JPEG; q=1"),
            Some(ImageFormat::Jpeg)
        );
        for refused in [
            "image/x-exr",
            "image/vnd.radiance",
            "image/vnd-ms.dds",
            "image/avif",
        ] {
            assert_eq!(image_format_for_mime(refused), None, "{refused}");
        }
        assert!(declared_image_format(None).is_err());
    }
}
