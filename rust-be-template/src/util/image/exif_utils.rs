use anyhow::{Result, anyhow};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use exif::{In, Tag};
use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Seek},
    path::Path,
};
use tracing::{debug, warn};

pub fn extract_exif_shot_at(image_bytes: &[u8]) -> Result<Option<DateTime<Utc>>> {
    let mut cursor = Cursor::new(image_bytes);

    extract_exif_shot_at_from_reader(&mut cursor)
}

/// Reads EXIF directly from a staged upload without cloning its encoded bytes.
pub fn extract_exif_shot_at_from_path(path: &Path) -> Result<Option<DateTime<Utc>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    extract_exif_shot_at_from_reader(&mut reader)
}

fn extract_exif_shot_at_from_reader<R>(reader: &mut R) -> Result<Option<DateTime<Utc>>>
where
    R: BufRead + Seek,
{
    let exif_reader = match exif::Reader::new().read_from_container(reader) {
        Ok(r) => r,
        Err(e) => {
            debug!(error = %e, "Could not read standard EXIF container");
            return Ok(None);
        }
    };

    let field = exif_reader
        .get_field(Tag::DateTimeOriginal, In::PRIMARY)
        .or_else(|| exif_reader.get_field(Tag::DateTime, In::PRIMARY));

    let Some(field) = field else {
        warn!("EXIF present but no DateTime tags found");
        return Ok(None);
    };

    let raw = field.display_value().to_string();
    debug!(raw_exif_datetime = %raw, "Found raw EXIF metadata string");

    let clean_raw = raw
        .trim_matches('"')
        .replace('T', " ")
        .replace(['-', '/'], ":");

    // Now the string should look like "2024:11:27 23:39:00" regardless of input
    let parts: Vec<&str> = clean_raw.split_whitespace().collect();

    if parts.len() != 2 {
        warn!(
            clean_raw = %clean_raw,
            raw_exif_datetime = %raw,
            part_count = parts.len(),
            "Parsed datetime string does not have 2 parts"
        );
        return Ok(None);
    }

    let date_parts: Vec<&str> = parts[0].split(':').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();

    if date_parts.len() != 3 || time_parts.len() != 3 {
        // This is where your error was happening.
        // Now that we replaced '-' with ':', date_parts.len() should be 3.
        warn!(
            date_part_count = date_parts.len(),
            time_part_count = time_parts.len(),
            clean_raw = %clean_raw,
            "EXIF datetime parts invalid structure"
        );
        return Ok(None);
    }

    // Attempt to parse numbers
    let year: i32 = date_parts[0].parse().map_err(|_| anyhow!("Invalid year"))?;
    let month: u32 = date_parts[1]
        .parse()
        .map_err(|_| anyhow!("Invalid month"))?;
    let day: u32 = date_parts[2].parse().map_err(|_| anyhow!("Invalid day"))?;
    let hour: u32 = time_parts[0].parse().map_err(|_| anyhow!("Invalid hour"))?;
    let minute: u32 = time_parts[1]
        .parse()
        .map_err(|_| anyhow!("Invalid minute"))?;
    let second: u32 = time_parts[2]
        .parse()
        .map_err(|_| anyhow!("Invalid second"))?;

    let date =
        NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| anyhow!("Date out of range"))?;
    let time = NaiveTime::from_hms_opt(hour, minute, second)
        .ok_or_else(|| anyhow!("Time out of range"))?;

    let naive_dt = NaiveDateTime::new(date, time);
    let dt_utc: DateTime<Utc> = Utc.from_utc_datetime(&naive_dt);

    Ok(Some(dt_utc))
}

#[cfg(test)]
mod tests {
    use std::io::{BufWriter, Cursor, Write};

    use chrono::{TimeZone, Utc};
    use exif::{Field, In, Tag, Value, experimental::Writer};
    use image::{ImageEncoder, Rgba, RgbaImage, codecs::avif::AvifEncoder};

    use super::extract_exif_shot_at_from_path;

    /// A little-endian TIFF block holding only the capture time, as it sits
    /// inside an image's EXIF item.
    fn exif_with_capture_time(capture_time: &str) -> anyhow::Result<Vec<u8>> {
        let field = Field {
            tag: Tag::DateTimeOriginal,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![capture_time.as_bytes().to_vec()]),
        };
        let mut writer = Writer::new();
        writer.push_field(&field);
        let mut block = Cursor::new(Vec::new());
        writer.write(&mut block, true)?;
        Ok(block.into_inner())
    }

    #[test]
    fn capture_time_is_read_from_an_avif_container() -> anyhow::Result<()> {
        let file = tempfile::Builder::new().suffix(".avif").tempfile()?;
        let image = RgbaImage::from_pixel(16, 16, Rgba([90, 60, 30, 255]));
        let mut output = BufWriter::new(file.reopen()?);
        let mut encoder = AvifEncoder::new_with_speed_quality(&mut output, 10, 80);
        encoder
            .set_exif_metadata(exif_with_capture_time("2026:05:17 14:03:21")?)
            .map_err(|error| anyhow::anyhow!("AVIF encoder refused EXIF: {error}"))?;
        encoder.write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )?;
        output.flush()?;
        drop(output);

        let expected = Utc.with_ymd_and_hms(2026, 5, 17, 14, 3, 21).single();
        assert!(expected.is_some());
        assert_eq!(extract_exif_shot_at_from_path(file.path())?, expected);
        Ok(())
    }

    #[test]
    fn an_avif_without_exif_has_no_capture_time() -> anyhow::Result<()> {
        let file = tempfile::Builder::new().suffix(".avif").tempfile()?;
        RgbaImage::from_pixel(16, 16, Rgba([90, 60, 30, 255]))
            .save_with_format(file.path(), image::ImageFormat::Avif)?;
        assert_eq!(extract_exif_shot_at_from_path(file.path())?, None);
        Ok(())
    }
}
