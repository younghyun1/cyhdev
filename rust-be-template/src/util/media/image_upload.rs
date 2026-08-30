//! Image-upload metadata validation shared by media endpoints.

/// MIME types accepted by the image decoder.
pub const ALLOWED_IMAGE_MIME_TYPES: [&str; 16] = [
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/x-portable-anymap",
    "image/tiff",
    "image/x-tga",
    "image/vnd-ms.dds",
    "image/bmp",
    "image/vnd.microsoft.icon",
    "image/vnd.radiance",
    "image/x-exr",
    "image/farbfeld",
    "image/avif",
    "image/qoi",
    "image/vnd.zbrush.pcx",
];

/// Returns whether the declared MIME type is supported for image uploads.
pub fn is_allowed_image_mime(content_type: &str) -> bool {
    ALLOWED_IMAGE_MIME_TYPES.contains(&content_type)
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
    use super::{has_file_extension, is_allowed_image_mime};

    #[test]
    fn validates_declared_image_metadata() {
        assert!(is_allowed_image_mime("image/jpeg"));
        assert!(!is_allowed_image_mime("application/pdf"));
        assert!(has_file_extension("photo.jpeg"));
        assert!(!has_file_extension("photo"));
        assert!(!has_file_extension("photo."));
    }
}
