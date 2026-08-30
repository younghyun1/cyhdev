//! Image output variants and the common AVIF encoding contract.

use image::ImageFormat;

pub const IMAGE_ENCODING_FORMAT: ImageFormat = ImageFormat::Avif;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CyhdevImageType {
    ProfilePicture,
    Photograph,
    Thumbnail,
    DemoThumbnail,
}

impl CyhdevImageType {
    pub fn max_long_width(&self) -> u32 {
        match self {
            CyhdevImageType::ProfilePicture => 400,
            CyhdevImageType::Photograph => 6000,
            CyhdevImageType::Thumbnail => 800,
            CyhdevImageType::DemoThumbnail => 512,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CyhdevImageType::ProfilePicture => "profile_picture",
            CyhdevImageType::Photograph => "photograph",
            CyhdevImageType::Thumbnail => "thumbnail",
            CyhdevImageType::DemoThumbnail => "demo_thumbnail",
        }
    }
}

pub fn format_size(bytes: usize) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;

    let bytes_float = bytes as f64;
    if bytes_float < KIB {
        format!("{bytes} B")
    } else if bytes_float < MIB {
        format!("{:.2} KiB", bytes_float / KIB)
    } else if bytes_float < GIB {
        format!("{:.2} MiB", bytes_float / MIB)
    } else {
        format!("{:.2} GiB", bytes_float / GIB)
    }
}
