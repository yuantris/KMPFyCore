use serde::{Deserialize, Serialize};

use crate::StitchError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StitchMode {
    Scroll,
    Panorama,
    Vertical,
    Horizontal,
    Grid,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StitchConfig {
    pub mode: StitchMode,
    /// Maximum output width; 0 means use the calculated width.
    pub max_width: u32,
    /// Maximum output height; 0 means use the calculated height.
    pub max_height: u32,
    pub min_overlap_ratio: f32,
    pub max_overlap_ratio: f32,
    pub min_confidence: f32,
    /// Automatically detect and remove common fixed top/bottom bars before matching.
    pub auto_detect_fixed_bars: bool,
    /// Explicit fixed header crop in pixels, applied after automatic detection.
    pub fixed_header_height: u32,
    /// Explicit fixed footer crop in pixels, applied after automatic detection.
    pub fixed_footer_height: u32,
    /// Per-input manual crop [top, bottom] in source pixels, indexed by input order.
    pub manual_crops: Vec<[u32; 2]>,
    /// Signed pixel offset applied to each seam; positive moves the next image down.
    pub seam_offsets: Vec<i32>,
    /// Per-image vertical translation in output pixels, independent of overlap cropping.
    #[serde(default)]
    pub image_offsets: Vec<i32>,
    pub blend: bool,
    pub output_format: String,
    /// Grid columns; 0 selects a near-square grid.
    pub columns: u32,
    pub spacing: u32,
    pub background: [u8; 4],
    /// Hard output pixel budget to prevent accidental huge allocations.
    pub max_output_pixels: u64,
}

impl Default for StitchConfig {
    fn default() -> Self {
        Self {
            mode: StitchMode::Scroll,
            max_width: 0,
            max_height: 0,
            min_overlap_ratio: 0.05,
            max_overlap_ratio: 0.95,
            min_confidence: 0.45,
            auto_detect_fixed_bars: true,
            fixed_header_height: 0,
            fixed_footer_height: 0,
            manual_crops: Vec::new(),
            seam_offsets: Vec::new(),
            image_offsets: Vec::new(),
            blend: true,
            output_format: "png".into(),
            columns: 0,
            spacing: 12,
            background: [255, 255, 255, 255],
            max_output_pixels: 100_000_000,
        }
    }
}

impl StitchConfig {
    pub fn validate(&self) -> Result<(), StitchError> {
        if !self.min_overlap_ratio.is_finite()
            || !self.max_overlap_ratio.is_finite()
            || self.min_overlap_ratio < 0.0
            || self.max_overlap_ratio > 1.0
            || self.min_overlap_ratio > self.max_overlap_ratio
        {
            return Err(StitchError::InvalidConfig("overlap ratios must satisfy 0 <= min <= max <= 1".into()));
        }
        if !self.min_confidence.is_finite() || !(0.0..=1.0).contains(&self.min_confidence) {
            return Err(StitchError::InvalidConfig("min_confidence must be between 0 and 1".into()));
        }
        if self.max_output_pixels == 0 {
            return Err(StitchError::InvalidConfig("max_output_pixels must be greater than zero".into()));
        }
        let format = self.output_format.to_ascii_lowercase();
        if !matches!(format.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp") {
            return Err(StitchError::InvalidConfig(format!("unsupported output format: {}", self.output_format)));
        }
        Ok(())
    }
}
