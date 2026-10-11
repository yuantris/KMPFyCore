//! Cross-platform image stitching engine.
//!
//! The first implementation provides deterministic layout stitching and a
//! pure-Rust overlap matcher for vertical scrolling screenshots. Panorama
//! registration is intentionally kept behind a future optional OpenCV backend.

mod config;
mod error;
mod layout;
mod scroll;

pub use config::{StitchConfig, StitchMode};
pub use error::StitchError;
pub use layout::stitch_layout;
pub use scroll::stitch_scroll;

use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StitchSegment {
    pub image_index: usize,
    pub x: i64,
    pub y: i64,
    pub crop_x: u32,
    pub crop_y: u32,
    pub crop_width: u32,
    pub crop_height: u32,
    /// Normalized matcher confidence in [0, 1]; 1 for deterministic layouts.
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StitchMetadata {
    pub width: u32,
    pub height: u32,
    pub image_count: usize,
    pub mode: StitchMode,
    pub segments: Vec<StitchSegment>,
}

pub struct StitchResult {
    pub image: RgbaImage,
    pub metadata: StitchMetadata,
}

pub fn stitch_images(
    images: &[RgbaImage],
    config: &StitchConfig,
) -> Result<StitchResult, StitchError> {
    if images.is_empty() {
        return Err(StitchError::EmptyInput);
    }
    config.validate()?;
    match config.mode {
        StitchMode::Vertical | StitchMode::Horizontal | StitchMode::Grid => {
            stitch_layout(images, config)
        }
        StitchMode::Scroll => stitch_scroll(images, config),
        StitchMode::Panorama => Err(StitchError::OpenCvUnavailable),
        StitchMode::Auto => {
            // Conservative initial auto mode: do not silently apply geometric
            // panorama transforms; try the screenshot matcher first.
            stitch_scroll(images, config)
        }
    }
}

/// Load files, stitch, encode, and save the result. Suitable for JNI/FFI calls.
pub fn stitch_files(
    input_paths: &[PathBuf],
    output_path: &Path,
    config: &StitchConfig,
) -> Result<StitchMetadata, StitchError> {
    if input_paths.is_empty() {
        return Err(StitchError::EmptyInput);
    }
    let images = input_paths.iter()
        .map(core_image::decode_file)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| StitchError::Image(error.to_string()))?;
    let result = stitch_images(&images, config)?;
    let format = output_path.extension()
        .and_then(|value| value.to_str())
        .unwrap_or(&config.output_format);
    core_image::encode_file(&result.image, output_path, format)
        .map_err(|error| StitchError::Image(error.to_string()))?;
    Ok(result.metadata)
}
