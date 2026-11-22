//! Turns one staged photograph into its stored main image and thumbnail.
//!
//! Every format except AVIF is decoded and encoded to AVIF, which drops the
//! file's embedded metadata. An AVIF upload is instead stored byte-for-byte,
//! so its EXIF (capture time, GPS position, camera and lens), XMP, colour
//! profile, bit depth, and HDR signalling reach the object store unchanged.
//!
//! The server has no AV1 decoder, so it cannot derive a thumbnail from an AVIF
//! original. The client renders the photograph to a small image in a format
//! the server does decode and sends it in the same request; that rendering is
//! the thumbnail's only source.

use std::path::{Path, PathBuf};

use anyhow::anyhow;

use crate::util::{
    image::{
        avif_original::require_avif_signature,
        image_variant::CyhdevImageType,
        process_uploaded_image_files::{ProcessedImageFile, process_uploaded_image_files},
    },
    media::image_upload::{declared_image_format, is_avif_mime},
};

/// A client-rendered image that stands in for an AVIF original when the
/// thumbnail is produced.
pub struct StagedThumbnailSource<'a> {
    pub path: &'a Path,
    pub content_type: Option<&'a str>,
}

/// One staged photograph and, for an AVIF original, its thumbnail source.
pub struct StagedPhotograph<'a> {
    pub source: &'a Path,
    pub content_type: Option<&'a str>,
    pub thumbnail_source: Option<StagedThumbnailSource<'a>>,
}

/// The file uploaded as the full-size photograph object.
pub enum MainImage {
    /// The source was decoded and encoded to AVIF.
    Encoded(ProcessedImageFile),
    /// The staged AVIF source itself. The caller owns that file and must keep
    /// it until the object has been uploaded.
    Original(PathBuf),
}

impl MainImage {
    pub fn path_buf(&self) -> PathBuf {
        match self {
            Self::Encoded(file) => file.path_buf(),
            Self::Original(path) => path.clone(),
        }
    }

    /// Whether the main object is the staged source rather than a new encode.
    pub fn is_original(&self) -> bool {
        matches!(self, Self::Original(_))
    }
}

/// Both stored objects of a photograph; each is an AVIF file.
pub struct PreparedPhotograph {
    pub main: MainImage,
    pub thumbnail: ProcessedImageFile,
}

/// Produces the main image and thumbnail for one staged photograph.
///
/// A thumbnail source is required with an AVIF photograph and refused with any
/// other type, so a request cannot make the thumbnail differ from a photograph
/// the server is able to decode itself.
pub async fn prepare_photograph(
    staged: StagedPhotograph<'_>,
) -> anyhow::Result<PreparedPhotograph> {
    if staged.content_type.is_some_and(is_avif_mime) {
        prepare_avif_original(staged).await
    } else {
        prepare_encoded(staged).await
    }
}

async fn prepare_avif_original(staged: StagedPhotograph<'_>) -> anyhow::Result<PreparedPhotograph> {
    let Some(thumbnail_source) = staged.thumbnail_source else {
        return Err(anyhow!(
            "An AVIF photograph requires a rendered thumbnail image"
        ));
    };
    let format = declared_image_format(thumbnail_source.content_type)?;
    let original = staged.source.to_path_buf();
    let probe_path = original.clone();
    match tokio::task::spawn_blocking(move || require_avif_signature(&probe_path)).await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => return Err(error),
        Err(error) => return Err(anyhow!("AVIF signature check task failed: {error}")),
    }
    let mut outputs = process_uploaded_image_files(
        thumbnail_source.path,
        format,
        vec![CyhdevImageType::Thumbnail],
    )
    .await?
    .into_iter();
    let Some(thumbnail) = outputs.next() else {
        return Err(anyhow!("encoder produced no thumbnail"));
    };
    Ok(PreparedPhotograph {
        main: MainImage::Original(original),
        thumbnail,
    })
}

async fn prepare_encoded(staged: StagedPhotograph<'_>) -> anyhow::Result<PreparedPhotograph> {
    if staged.thumbnail_source.is_some() {
        return Err(anyhow!(
            "A rendered thumbnail image is only accepted with an AVIF photograph"
        ));
    }
    let format = declared_image_format(staged.content_type)?;
    let mut outputs = process_uploaded_image_files(
        staged.source,
        format,
        vec![CyhdevImageType::Photograph, CyhdevImageType::Thumbnail],
    )
    .await?
    .into_iter();
    let Some(main) = outputs.next() else {
        return Err(anyhow!("encoder produced no main image"));
    };
    let Some(thumbnail) = outputs.next() else {
        return Err(anyhow!("encoder produced no thumbnail"));
    };
    Ok(PreparedPhotograph {
        main: MainImage::Encoded(main),
        thumbnail,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use image::{ImageFormat, Rgb, RgbImage};
    use tempfile::NamedTempFile;

    use super::{StagedPhotograph, StagedThumbnailSource, prepare_photograph};

    fn write_image(
        width: u32,
        height: u32,
        format: ImageFormat,
        suffix: &str,
    ) -> anyhow::Result<NamedTempFile> {
        let file = tempfile::Builder::new().suffix(suffix).tempfile()?;
        // RGB rather than RGBA: the JPEG encoder has no alpha channel.
        RgbImage::from_pixel(width, height, Rgb([40, 90, 200]))
            .save_with_format(file.path(), format)?;
        Ok(file)
    }

    fn is_avif(path: &Path) -> anyhow::Result<bool> {
        Ok(std::fs::read(path)?.get(4..12) == Some(b"ftypavif".as_slice()))
    }

    #[tokio::test]
    async fn avif_original_is_stored_untouched_with_a_thumbnail_from_the_rendering()
    -> anyhow::Result<()> {
        let original = write_image(64, 48, ImageFormat::Avif, ".avif")?;
        let original_bytes = std::fs::read(original.path())?;
        let rendering = write_image(96, 72, ImageFormat::Jpeg, ".jpg")?;
        let prepared = prepare_photograph(StagedPhotograph {
            source: original.path(),
            content_type: Some("image/avif"),
            thumbnail_source: Some(StagedThumbnailSource {
                path: rendering.path(),
                content_type: Some("image/jpeg"),
            }),
        })
        .await?;

        assert!(prepared.main.is_original());
        assert_eq!(prepared.main.path_buf(), original.path());
        assert_eq!(std::fs::read(prepared.main.path_buf())?, original_bytes);
        assert!(is_avif(prepared.thumbnail.path())?);
        let thumbnail = std::fs::read(prepared.thumbnail.path())?;
        assert_ne!(thumbnail, original_bytes);
        Ok(())
    }

    #[tokio::test]
    async fn avif_original_needs_a_decodable_thumbnail_source_and_a_real_signature()
    -> anyhow::Result<()> {
        let original = write_image(64, 48, ImageFormat::Avif, ".avif")?;
        let rendering = write_image(32, 24, ImageFormat::Png, ".png")?;
        let missing = prepare_photograph(StagedPhotograph {
            source: original.path(),
            content_type: Some("image/avif"),
            thumbnail_source: None,
        })
        .await;
        assert!(missing.is_err());

        // A second AVIF cannot serve as the thumbnail source: nothing decodes it.
        let undecodable = prepare_photograph(StagedPhotograph {
            source: original.path(),
            content_type: Some("image/avif"),
            thumbnail_source: Some(StagedThumbnailSource {
                path: original.path(),
                content_type: Some("image/avif"),
            }),
        })
        .await;
        assert!(undecodable.is_err());

        // A PNG declared as AVIF would otherwise be published as `image/avif`.
        let mislabelled = prepare_photograph(StagedPhotograph {
            source: rendering.path(),
            content_type: Some("image/avif"),
            thumbnail_source: Some(StagedThumbnailSource {
                path: rendering.path(),
                content_type: Some("image/png"),
            }),
        })
        .await;
        assert!(mislabelled.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn other_formats_are_encoded_and_refuse_a_thumbnail_source() -> anyhow::Result<()> {
        let source = write_image(48, 32, ImageFormat::Png, ".png")?;
        let prepared = prepare_photograph(StagedPhotograph {
            source: source.path(),
            content_type: Some("image/png"),
            thumbnail_source: None,
        })
        .await?;
        assert!(!prepared.main.is_original());
        assert_ne!(prepared.main.path_buf(), source.path());
        assert!(is_avif(&prepared.main.path_buf())?);
        assert!(is_avif(prepared.thumbnail.path())?);

        let rendering = write_image(32, 24, ImageFormat::Jpeg, ".jpg")?;
        let unexpected = prepare_photograph(StagedPhotograph {
            source: source.path(),
            content_type: Some("image/png"),
            thumbnail_source: Some(StagedThumbnailSource {
                path: rendering.path(),
                content_type: Some("image/jpeg"),
            }),
        })
        .await;
        assert!(unexpected.is_err());
        Ok(())
    }
}
