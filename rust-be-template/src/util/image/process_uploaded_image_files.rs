//! Bounded, file-backed image decoding and encoding.

use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::anyhow;
use fast_image_resize::{PixelType, ResizeOptions, Resizer, images::Image as FastImage};
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader, Limits};
use tempfile::TempPath;
use tokio::sync::Semaphore;
use tracing::info;

use super::image_variant::{CyhdevImageType, IMAGE_ENCODING_FORMAT, format_size};

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

/// Decodes one staged source and writes ordered, progressively smaller variants.
///
/// Callers should request variants from largest to smallest. The decoded source
/// is consumed by each resize, so a photograph and thumbnail never require two
/// decoded originals at once. Encoded bytes are written through a fixed buffer.
pub async fn process_uploaded_image_files(
    source: &Path,
    format: Option<ImageFormat>,
    variants: Vec<CyhdevImageType>,
) -> anyhow::Result<Vec<ProcessedImageFile>> {
    validate_variant_order(&variants)?;
    let permit = IMAGE_PROCESSING_PERMITS
        .acquire()
        .await
        .map_err(|error| anyhow!("Image processing limiter closed: {error}"))?;
    let source = source.to_path_buf();
    let result = tokio::task::spawn_blocking(move || process_files(&source, format, variants))
        .await
        .map_err(|error| anyhow!("Blocking image processing task panicked: {error}"))?;
    drop(permit);
    result
}

fn process_files(
    source: &Path,
    format: Option<ImageFormat>,
    variants: Vec<CyhdevImageType>,
) -> anyhow::Result<Vec<ProcessedImageFile>> {
    let start = Instant::now();
    let original_size = source.metadata()?.len();
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

fn open_reader(
    source: &Path,
    format: Option<ImageFormat>,
) -> anyhow::Result<ImageReader<BufReader<File>>> {
    let file = File::open(source)?;
    let reader = ImageReader::new(BufReader::with_capacity(FILE_BUFFER_BYTES, file));
    match format {
        Some(format) => {
            let mut reader = reader;
            reader.set_format(format);
            Ok(reader)
        }
        None => reader.with_guessed_format().map_err(anyhow::Error::from),
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
    image.write_to(&mut writer, IMAGE_ENCODING_FORMAT)?;
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
    use super::{CyhdevImageType, validate_dimensions, validate_variant_order};

    #[test]
    fn rejects_pixel_bombs_and_inverted_variant_order() {
        assert!(validate_dimensions((65_536, 65_536)).is_err());
        assert!(
            validate_variant_order(&[CyhdevImageType::Thumbnail, CyhdevImageType::Photograph,])
                .is_err()
        );
    }
}
