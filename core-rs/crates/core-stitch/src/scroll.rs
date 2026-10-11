use image::{imageops, Rgba, RgbaImage};

use crate::{
    config::{StitchConfig, StitchMode},
    error::StitchError,
    StitchMetadata, StitchResult, StitchSegment,
};

/// Pure-Rust matcher for consecutive vertical scrolling screenshots.
///
/// The matcher compares candidate bottom/top overlap bands using normalized
/// mean absolute grayscale error. It intentionally fails on low confidence
/// rather than generating a misleading long screenshot.
pub fn stitch_scroll(
    images: &[RgbaImage],
    config: &StitchConfig,
) -> Result<StitchResult, StitchError> {
    if images.is_empty() {
        return Err(StitchError::EmptyInput);
    }
    config.validate()?;

    let target_width = images[0].width();
    if target_width == 0 || images.iter().any(|image| image.width() == 0 || image.height() == 0) {
        return Err(StitchError::InvalidDimensions);
    }

    let normalized: Vec<RgbaImage> = images.iter().map(|image| {
        if image.width() == target_width {
            image.clone()
        } else {
            let height = (u64::from(image.height()) * u64::from(target_width)
                / u64::from(image.width())).max(1) as u32;
            imageops::resize(image, target_width, height, imageops::FilterType::Lanczos3)
        }
    }).collect();

    let mut overlaps = Vec::with_capacity(normalized.len().saturating_sub(1));
    let mut confidences = Vec::with_capacity(normalized.len().saturating_sub(1));
    for index in 0..normalized.len().saturating_sub(1) {
        let (overlap, confidence) = best_overlap(
            &normalized[index],
            &normalized[index + 1],
            config.min_overlap_ratio,
            config.max_overlap_ratio,
        );
        if confidence < config.min_confidence {
            return Err(StitchError::MatchFailed {
                left: index,
                right: index + 1,
                confidence,
            });
        }
        overlaps.push(overlap);
        confidences.push(confidence);
    }

    let spacing = 0u64;
    let mut y_positions = Vec::with_capacity(normalized.len());
    let mut y = 0u64;
    for (index, image) in normalized.iter().enumerate() {
        y_positions.push(y);
        if index + 1 < normalized.len() {
            let overlap = u64::from(overlaps[index]);
            y = y.checked_add(u64::from(image.height()).saturating_sub(overlap))
                .and_then(|v| v.checked_add(spacing))
                .ok_or(StitchError::OutputTooLarge)?;
        }
    }

    let width = u64::from(target_width);
    let height = y.checked_add(u64::from(normalized.last().ok_or(StitchError::EmptyInput)?.height()))
        .ok_or(StitchError::OutputTooLarge)?;
    let (canvas_width, canvas_height, scale) = bounded_dimensions(width, height, config)?;
    let mut canvas = RgbaImage::from_pixel(
        canvas_width,
        canvas_height,
        Rgba(config.background),
    );
    let mut segments = Vec::with_capacity(normalized.len());

    for (index, image) in normalized.iter().enumerate() {
        let crop_y = if index == 0 { 0 } else { overlaps[index - 1] };
        let source_height = image.height().saturating_sub(crop_y);
        let remaining = (canvas_height as f64 / scale).ceil() as u64;
        let y_source = y_positions[index].saturating_add(u64::from(crop_y));
        let visible_height = u64::from(source_height).min(remaining.saturating_sub(y_source));
        if visible_height > 0 {
            let crop = imageops::crop_imm(image, 0, crop_y, image.width(), visible_height as u32).to_image();
            let x = 0u32;
            let y_dest = (y_source as f64 * scale).round().max(0.0) as u32;
            let resized = if (scale - 1.0).abs() > f64::EPSILON {
                imageops::resize(
                    &crop,
                    ((f64::from(crop.width()) * scale).round() as u32).max(1),
                    ((f64::from(crop.height()) * scale).round() as u32).max(1),
                    imageops::FilterType::Lanczos3,
                )
            } else {
                crop
            };
            if y_dest < canvas.height() {
                imageops::overlay(&mut canvas, &resized, i64::from(x), i64::from(y_dest));
            }
        }
        segments.push(StitchSegment {
            image_index: index,
            x: 0,
            y: (y_positions[index] as f64 * scale) as i64,
            crop_x: 0,
            crop_y,
            crop_width: image.width(),
            crop_height: source_height,
            confidence: if index == 0 { 1.0 } else { confidences[index - 1] },
        });
    }

    Ok(StitchResult {
        metadata: StitchMetadata {
            width: canvas.width(),
            height: canvas.height(),
            image_count: images.len(),
            mode: StitchMode::Scroll,
            segments,
        },
        image: canvas,
    })
}

fn best_overlap(
    upper: &RgbaImage,
    lower: &RgbaImage,
    min_ratio: f32,
    max_ratio: f32,
) -> (u32, f32) {
    let max_height = upper.height().min(lower.height());
    if max_height < 2 {
        return (0, 0.0);
    }
    let min_overlap = ((max_height as f32 * min_ratio).ceil() as u32)
        .max(1)
        .min(max_height.saturating_sub(1));
    let max_overlap = ((max_height as f32 * max_ratio).floor() as u32)
        .min(max_height.saturating_sub(1))
        .max(min_overlap);
    let x_step = (upper.width() / 96).max(1);
    let y_step = 1u32.max(max_overlap / 128);
    let mut best = (min_overlap, f32::NEG_INFINITY);

    let mut overlap = min_overlap;
    while overlap <= max_overlap {
        let upper_start = upper.height() - overlap;
        let mut error_sum = 0.0f64;
        let mut samples = 0u64;
        let mut y = 0;
        while y < overlap {
            let mut x = 0;
            while x < upper.width() {
                let a = luminance(upper.get_pixel(x, upper_start + y).0);
                let b = luminance(lower.get_pixel(x, y).0);
                error_sum += (a - b).abs() as f64 / 255.0;
                samples += 1;
                x = x.saturating_add(x_step);
            }
            y = y.saturating_add(y_step);
        }
        let error = if samples == 0 { 1.0 } else { error_sum / samples as f64 };
        // Prefer the lowest error. The slight penalty for very large overlaps
        // reduces false matches in repeated/flat backgrounds.
        let score = (1.0 - error as f32) - (overlap as f32 / max_height as f32) * 0.015;
        if score > best.1 {
            best = (overlap, score);
        }
        overlap = overlap.saturating_add(y_step);
        if overlap == u32::MAX { break; }
    }

    (best.0, best.1.clamp(0.0, 1.0))
}

fn luminance(pixel: [u8; 4]) -> f32 {
    (0.2126 * f32::from(pixel[0]) + 0.7152 * f32::from(pixel[1]) + 0.0722 * f32::from(pixel[2]))
}

fn bounded_dimensions(width: u64, height: u64, config: &StitchConfig) -> Result<(u32, u32, f64), StitchError> {
    if width == 0 || height == 0 {
        return Err(StitchError::InvalidDimensions);
    }
    let mut scale = 1.0f64;
    if config.max_width > 0 && width > u64::from(config.max_width) {
        scale = scale.min(f64::from(config.max_width) / width as f64);
    }
    if config.max_height > 0 && height > u64::from(config.max_height) {
        scale = scale.min(f64::from(config.max_height) / height as f64);
    }
    let mut w = ((width as f64 * scale).floor() as u64).max(1);
    let mut h = ((height as f64 * scale).floor() as u64).max(1);
    let pixels = w.checked_mul(h).ok_or(StitchError::OutputTooLarge)?;
    if pixels > config.max_output_pixels {
        scale *= (config.max_output_pixels as f64 / pixels as f64).sqrt();
        w = ((width as f64 * scale).floor() as u64).max(1);
        h = ((height as f64 * scale).floor() as u64).max(1);
    }
    if w > u32::MAX as u64 || h > u32::MAX as u64
        || w.checked_mul(h).is_none_or(|p| p > config.max_output_pixels)
    {
        return Err(StitchError::OutputTooLarge);
    }
    Ok((w as u32, h as u32, scale))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_overlapping_rows_are_detected() {
        let mut a = RgbaImage::new(8, 20);
        let mut b = RgbaImage::new(8, 20);
        for y in 0..20 {
            for x in 0..8 {
                let color = Rgba([(y * 7) as u8, (x * 13) as u8, ((x + y) * 5) as u8, 255]);
                a.put_pixel(x, y, color);
                b.put_pixel(x, y, color);
            }
        }
        // Make the lower image's top 8 rows match the upper image's bottom 8 rows.
        for y in 0..8 {
            for x in 0..8 {
                b.put_pixel(x, y, *a.get_pixel(x, 12 + y));
            }
        }
        let (overlap, confidence) = best_overlap(&a, &b, 0.2, 0.8);
        assert_eq!(overlap, 8);
        assert!(confidence > 0.9);
    }
}
