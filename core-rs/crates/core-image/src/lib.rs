//! Shared image I/O and safe canvas helpers for core-rs.
//!
//! Platform layers should convert platform-specific URIs to readable files or
//! byte buffers before calling this crate.

use image::{DynamicImage, ImageFormat, RgbaImage};
use std::io::Cursor;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("image input is empty")]
    EmptyInput,
    #[error("failed to decode image: {0}")]
    Decode(#[from] image::ImageError),
    #[error("unsupported image output format: {0}")]
    UnsupportedFormat(String),
    #[error("image dimensions are invalid or too large")]
    InvalidDimensions,
    #[error("failed to encode image: {0}")]
    Encode(String),
}

pub const DEFAULT_MAX_PIXELS: u64 = 100_000_000;

/// Decode an encoded image into the canonical RGBA pixel format.
pub fn decode(bytes: &[u8]) -> Result<RgbaImage, ImageError> {
    if bytes.is_empty() {
        return Err(ImageError::EmptyInput);
    }
    let decoded = image::load_from_memory(bytes)?;
    validate_dimensions(decoded.width(), decoded.height(), DEFAULT_MAX_PIXELS)?;
    Ok(decoded.to_rgba8())
}

/// Decode an image from a path, checking its dimensions before returning pixels.
pub fn decode_file(path: impl AsRef<std::path::Path>) -> Result<RgbaImage, ImageError> {
    let decoded = image::ImageReader::open(path)
        .map_err(image::ImageError::IoError)?
        .with_guessed_format()?
        .decode()?;
    validate_dimensions(decoded.width(), decoded.height(), DEFAULT_MAX_PIXELS)?;
    Ok(decoded.to_rgba8())
}

/// Encode an RGBA image to PNG or JPEG bytes.
pub fn encode(image: &RgbaImage, format: &str) -> Result<Vec<u8>, ImageError> {
    validate_dimensions(image.width(), image.height(), DEFAULT_MAX_PIXELS)?;
    let image_format = match format.trim().to_ascii_lowercase().as_str() {
        "png" => ImageFormat::Png,
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "webp" => ImageFormat::WebP,
        "bmp" => ImageFormat::Bmp,
        other => return Err(ImageError::UnsupportedFormat(other.to_owned())),
    };
    let mut bytes = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(image.clone()).write_to(&mut bytes, image_format)
        .map_err(|error| ImageError::Encode(error.to_string()))?;
    Ok(bytes.into_inner())
}

/// Save an RGBA image to disk using an explicit supported format.
pub fn encode_file(
    image: &RgbaImage,
    path: impl AsRef<std::path::Path>,
    format: &str,
) -> Result<(), ImageError> {
    let bytes = encode(image, format)?;
    std::fs::write(path, bytes).map_err(|error| ImageError::Encode(error.to_string()))
}

/// Validate dimensions using a pixel budget before allocating large buffers.
pub fn validate_dimensions(width: u32, height: u32, max_pixels: u64) -> Result<(), ImageError> {
    if width == 0 || height == 0
        || u64::from(width).checked_mul(u64::from(height)).is_none_or(|pixels| pixels > max_pixels)
    {
        return Err(ImageError::InvalidDimensions);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_input() {
        assert!(matches!(decode(&[]), Err(ImageError::EmptyInput)));
    }

    #[test]
    fn rejects_oversized_dimensions() {
        assert!(validate_dimensions(10_000, 10_000, 10_000).is_err());
    }

    #[test]
    fn png_round_trip() {
        let image = RgbaImage::from_pixel(4, 3, image::Rgba([12, 34, 56, 255]));
        let encoded = encode(&image, "png").expect("encode");
        let decoded = decode(&encoded).expect("decode");
        assert_eq!(decoded.dimensions(), (4, 3));
        assert_eq!(decoded.get_pixel(0, 0).0, [12, 34, 56, 255]);
    }
}
