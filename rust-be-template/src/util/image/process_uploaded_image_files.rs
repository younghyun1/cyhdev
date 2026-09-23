//! Bounded, file-backed image decoding and encoding.

use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
    time::Instant,
};

use anyhow::anyhow;
use fast_image_resize::{PixelType, ResizeOptions, Resizer, images::Image as FastImage};
use image::{
    DynamicImage, GenericImageView, ImageFormat, ImageReader, Limits, codecs::avif::AvifEncoder,
};
use tempfile::TempPath;
use tokio::sync::Semaphore;
use tracing::info;

use super::image_variant::{AVIF_QUALITY, CyhdevImageType, format_size};

const FILE_BUFFER_BYTES: usize = 64 * 1024;
const MAX_DECODED_PIXELS: u64 = 64 * 1024 * 1024;
const MAX_DECODER_ALLOCATION_BYTES: u64 = 256 * 1024 * 1024;
const MAX_IMAGE_DIMENSION: u32 = 16_384;
const MAX_CONCURRENT_IMAGE_JOBS: usize = 2;

/// One encoded image held in an automatically removed temporary file.
pub struct ProcessedImageFile {
    path: TempPath,
    pub image_type: CyhdevImageType,
    pub size_bytes: u64,
}

impl ProcessedImageFile {
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    pub fn path_buf(&self) -> PathBuf {
        self.path.to_path_buf()
    }
}

static IMAGE_PROCESSING_PERMITS: Semaphore = Semaphore::const_new(MAX_CONCURRENT_IMAGE_JOBS);

/// Encoder threads per job: half the cores, so the two concurrent jobs
/// together use about every core and never oversubscribe the host that also
/// serves requests. Without a cap each encode would claim the whole pool.
static AVIF_ENCODER_THREADS: LazyLock<usize> = LazyLock::new(|| {
    std::thread::available_parallelism()
        .map(|cores| (cores.get() / 2).max(1))
        .unwrap_or(1)
});

/// Decodes one staged source and writes ordered, progressively smaller variants.
///
/// `format` is the decoder chosen from the upload's declared MIME type; the
/// file's signature must agree with it. Callers should request variants from
/// largest to smallest. The decoded source is consumed by each resize, so a
/// photograph and thumbnail never require two decoded originals at once.
/// Encoded bytes are written through a fixed buffer.
pub async fn process_uploaded_image_files(
    source: &Path,
    format: ImageFormat,
    variants: Vec<CyhdevImageType>,
) -> anyhow::Result<Vec<ProcessedImageFile>> {
    validate_variant_order(&variants)?;
    let permit = IMAGE_PROCESSING_PERMITS
        .acquire()
        .await
        .map_err(|error| anyhow!("Image processing limiter closed: {error}"))?;
    let source = source.to_path_buf();
    // The permit moves into the blocking closure: a cancelled request stops
    // awaiting, but the encode keeps running and must keep its slot.
    tokio::task::spawn_blocking(move || {
        let result = process_files(&source, format, variants);
        drop(permit);
        result
    })
    .await
    .map_err(|error| anyhow!("Blocking image processing task panicked: {error}"))?
}

fn process_files(
    source: &Path,
    format: ImageFormat,
    variants: Vec<CyhdevImageType>,
) -> anyhow::Result<Vec<ProcessedImageFile>> {
    let start = Instant::now();
    let original_size = source.metadata()?.len();
    require_matching_signature(source, format)?;
    let dimensions = open_reader(source, format)?.into_dimensions()?;
    validate_dimensions(dimensions)?;

    let mut reader = open_reader(source, format)?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(MAX_DECODER_ALLOCATION_BYTES);
    reader.limits(limits);
    let mut image = reader.decode()?;

    let mut processed = Vec::with_capacity(variants.len());
    for image_type in variants {
        image = resize_to_bound(image, image_type.max_long_width())?;
        let output = encode_to_temp_file(&image, image_type)?;
        info!(
            image_type = image_type.as_str(),
            original_size_bytes = original_size,
            original_size_human = %format_size(original_size as usize),
            processed_size_bytes = output.size_bytes,
            processed_size_human = %format_size(output.size_bytes as usize),
            elapsed_ms = %start.elapsed().as_millis(),
            "Completed file-backed image processing"
        );
        processed.push(output);
    }
    Ok(processed)
}

fn open_reader(source: &Path, format: ImageFormat) -> anyhow::Result<ImageReader<BufReader<File>>> {
    let file = File::open(source)?;
    let mut reader = ImageReader::new(BufReader::with_capacity(FILE_BUFFER_BYTES, file));
    reader.set_format(format);
    Ok(reader)
}

/// Rejects a file whose leading signature is not the declared format. Only
/// the magic bytes are read; no decoder runs for a mismatched upload.
fn require_matching_signature(source: &Path, declared: ImageFormat) -> anyhow::Result<()> {
    let file = File::open(source)?;
    let sniffed = ImageReader::new(BufReader::with_capacity(FILE_BUFFER_BYTES, file))
        .with_guessed_format()?
        .format();
    if sniffed == Some(declared) {
        Ok(())
    } else {
        Err(anyhow!(
            "Image content does not match its declared {declared:?} type"
        ))
    }
}

fn validate_dimensions((width, height): (u32, u32)) -> anyhow::Result<()> {
    let pixels = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or_else(|| anyhow!("Image dimensions overflow the decoded-pixel bound"))?;
    if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION || pixels > MAX_DECODED_PIXELS {
        return Err(anyhow!(
            "Image dimensions {width}x{height} exceed the decoded-pixel bound"
        ));
    }
    Ok(())
}

fn validate_variant_order(variants: &[CyhdevImageType]) -> anyhow::Result<()> {
    for pair in variants.windows(2) {
        if pair[0].max_long_width() < pair[1].max_long_width() {
            return Err(anyhow!(
                "Image variants must be ordered from largest to smallest"
            ));
        }
    }
    if variants.is_empty() {
        return Err(anyhow!("At least one image variant is required"));
    }
    Ok(())
}

fn resize_to_bound(image: DynamicImage, max_edge: u32) -> anyhow::Result<DynamicImage> {
    let (width, height) = image.dimensions();
    if width.max(height) <= max_edge {
        return Ok(image);
    }

    let scale = f64::from(max_edge) / f64::from(width.max(height));
    let new_width = (f64::from(width) * scale).round().max(1.0) as u32;
    let new_height = (f64::from(height) * scale).round().max(1.0) as u32;
    let source_data = image.into_rgba8().into_raw();
    let source = FastImage::from_vec_u8(width, height, source_data, PixelType::U8x4)
        .map_err(|error| anyhow!("Failed to create resize source: {error}"))?;
    let mut destination = FastImage::new(new_width, new_height, source.pixel_type());
    Resizer::new()
        .resize(&source, &mut destination, &ResizeOptions::default())
        .map_err(|error| anyhow!("Failed to resize image: {error}"))?;
    let buffer = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
        new_width,
        new_height,
        destination.into_vec(),
    )
    .ok_or_else(|| anyhow!("Failed to construct resized image buffer"))?;
    Ok(DynamicImage::ImageRgba8(buffer))
}

fn encode_to_temp_file(
    image: &DynamicImage,
    image_type: CyhdevImageType,
) -> anyhow::Result<ProcessedImageFile> {
    let named = tempfile::Builder::new()
        .prefix("cyhdev-processed-")
        .suffix(".avif")
        .tempfile()?;
    let (file, path) = named.into_parts();
    let mut writer = BufWriter::with_capacity(FILE_BUFFER_BYTES, file);
    let encoder =
        AvifEncoder::new_with_speed_quality(&mut writer, image_type.avif_speed(), AVIF_QUALITY)
            .with_num_threads(Some(*AVIF_ENCODER_THREADS));
    image.write_with_encoder(encoder)?;
    writer.flush()?;
    let size_bytes = writer.get_ref().metadata()?.len();
    drop(writer);
    Ok(ProcessedImageFile {
        path,
        image_type,
        size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use image::ImageFormat;

    use super::{
        AVIF_ENCODER_THREADS, CyhdevImageType, require_matching_signature, validate_dimensions,
        validate_variant_order,
    };

    #[test]
    fn signature_must_match_the_declared_format() -> anyhow::Result<()> {
        let mut png = tempfile::NamedTempFile::new()?;
        png.write_all(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR")?;
        assert!(require_matching_signature(png.path(), ImageFormat::Png).is_ok());
        assert!(require_matching_signature(png.path(), ImageFormat::Jpeg).is_err());
        let mut unknown = tempfile::NamedTempFile::new()?;
        unknown.write_all(b"not an image")?;
        assert!(require_matching_signature(unknown.path(), ImageFormat::Png).is_err());
        Ok(())
    }

    #[test]
    fn declared_png_encodes_to_avif_and_a_mislabelled_one_is_refused() -> anyhow::Result<()> {
        let source = tempfile::Builder::new().suffix(".png").tempfile()?;
        image::RgbaImage::from_pixel(32, 24, image::Rgba([200, 40, 90, 255]))
            .save_with_format(source.path(), ImageFormat::Png)?;
        let outputs = super::process_files(
            source.path(),
            ImageFormat::Png,
            vec![CyhdevImageType::Thumbnail],
        )?;
        let encoded = std::fs::read(
            outputs
                .first()
                .map(|output| output.path())
                .ok_or_else(|| anyhow::anyhow!("no thumbnail output"))?,
        )?;
        assert_eq!(encoded.get(4..12), Some(b"ftypavif".as_slice()));
        assert!(
            super::process_files(
                source.path(),
                ImageFormat::Jpeg,
                vec![CyhdevImageType::Thumbnail]
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn encoder_threads_stay_within_the_host() {
        let cores = std::thread::available_parallelism().map_or(1, |cores| cores.get());
        assert!(*AVIF_ENCODER_THREADS >= 1 && *AVIF_ENCODER_THREADS <= cores);
    }

    #[test]
    fn rejects_pixel_bombs_and_inverted_variant_order() {
        assert!(validate_dimensions((65_536, 65_536)).is_err());
        assert!(
            validate_variant_order(&[CyhdevImageType::Thumbnail, CyhdevImageType::Photograph,])
                .is_err()
        );
    }
}
