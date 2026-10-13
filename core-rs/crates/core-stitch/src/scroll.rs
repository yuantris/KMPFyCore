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

    let normalized: Vec<RgbaImage> = images.iter().enumerate().map(|(index, image)| {
        let resized = if image.width() == target_width {
            image.clone()
        } else {
            let height = (u64::from(image.height()) * u64::from(target_width)
                / u64::from(image.width())).max(1) as u32;
            imageops::resize(image, target_width, height, imageops::FilterType::Lanczos3)
        };
        let crop = config.manual_crops.get(index).copied().unwrap_or([0, 0]);
        let top = crop[0].min(resized.height().saturating_sub(1));
        let bottom = crop[1].min(resized.height().saturating_sub(top + 1));
        imageops::crop_imm(&resized, 0, top, resized.width(), resized.height() - top - bottom).to_image()
    }).collect();

    // Remove screen-fixed status/navigation bars before matching. These pixels
    // are not part of the scrolling document and otherwise bias overlap scores.
    let detected = if config.auto_detect_fixed_bars {
        detect_fixed_bars(&normalized)
    } else {
        (0, 0)
    };
    let header = if config.fixed_header_height > 0 {
        config.fixed_header_height
    } else {
        detected.0
    };
    let footer = if config.fixed_footer_height > 0 {
        config.fixed_footer_height
    } else {
        detected.1
    };
    let normalized: Vec<RgbaImage> = normalized.iter().map(|image| {
        let top = header.min(image.height().saturating_sub(1));
        let bottom = footer.min(image.height().saturating_sub(top + 1));
        imageops::crop_imm(image, 0, top, image.width(), image.height() - top - bottom).to_image()
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
            let automatic_step = i64::from(image.height().saturating_sub(overlaps[index]));
            let manual_offset = i64::from(config.seam_offsets.get(index).copied().unwrap_or(0));
            let current_image_offset = i64::from(config.image_offsets.get(index).copied().unwrap_or(0));
            let next_image_offset = i64::from(config.image_offsets.get(index + 1).copied().unwrap_or(0));
            let image_translation_delta = next_image_offset - current_image_offset;
            let adjusted_step = (automatic_step + manual_offset + image_translation_delta).max(1) as u64;
            y = y.checked_add(adjusted_step)
                .and_then(|v| v.checked_add(spacing))
                .ok_or(StitchError::OutputTooLarge)?;
        }
    }

    let width = u64::from(target_width);
    let last_index = normalized.len().saturating_sub(1);
    let last_image = normalized.last().ok_or(StitchError::EmptyInput)?;
    let last_crop = if last_index == 0 {
        0
    } else {
        (i64::from(overlaps[last_index - 1])
            + i64::from(config.seam_offsets.get(last_index - 1).copied().unwrap_or(0)))
            .clamp(0, i64::from(last_image.height().saturating_sub(1))) as u32
    };
    let height = y.checked_add(u64::from(last_image.height().saturating_sub(last_crop)))
        .ok_or(StitchError::OutputTooLarge)?;
    let (canvas_width, canvas_height, scale) = bounded_dimensions(width, height, config)?;
    let mut canvas = RgbaImage::from_pixel(
        canvas_width,
        canvas_height,
        Rgba(config.background),
    );
    let mut segments = Vec::with_capacity(normalized.len());

    for (index, image) in normalized.iter().enumerate() {
        // The next image is placed at y_positions[index]. Adjust its top crop
        // by the same seam delta: moving it down crops more of its top, moving
        // it up reveals more of its top. This keeps the seam contiguous.
        let crop_y = if index == 0 {
            0
        } else {
            let base = i64::from(overlaps[index - 1]);
            let delta = i64::from(config.seam_offsets.get(index - 1).copied().unwrap_or(0));
            (base + delta).clamp(0, i64::from(image.height().saturating_sub(1))) as u32
        };
        let source_height = image.height().saturating_sub(crop_y);
        let remaining = (canvas_height as f64 / scale).ceil() as u64;
        let y_source = y_positions[index];
        let visible_height = u64::from(source_height).min(remaining.saturating_sub(y_source));
        if visible_height > 0 {
            let crop = imageops::crop_imm(image, 0, crop_y, image.width(), visible_height as u32).to_image();
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
                imageops::overlay(&mut canvas, &resized, 0, i64::from(y_dest));
            }
        }
        segments.push(StitchSegment {
            image_index: index,
            x: 0,
            y: (y_positions[index] as f64 * scale) as i64,
            crop_x: 0,
            crop_y: crop_y.saturating_add(header).saturating_add(config.manual_crops.get(index).map(|crop| crop[0]).unwrap_or(0)),
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


/// Detect common fixed rows at the top and bottom of consecutive screenshots.
/// Only long, nearly identical edge bands are treated as chrome; short accidental
/// matches in repeated content are ignored.
fn detect_fixed_bars(images: &[RgbaImage]) -> (u32, u32) {
    if images.len() < 2 {
        return (0, 0);
    }
    let height = images.iter().map(RgbaImage::height).min().unwrap_or(0);
    let width = images.iter().map(RgbaImage::width).min().unwrap_or(0);
    if height < 40 || width == 0 {
        return (0, 0);
    }
    let limit = (height / 6).max(1);
    let row_matches = |y: u32| -> bool {
        let mut error = 0.0f64;
        let mut count = 0u64;
        let x_step = (width / 128).max(1);
        for image in images.iter().skip(1) {
            for x in (0..width).step_by(x_step as usize) {
                let a = images[0].get_pixel(x, y).0;
                let b = image.get_pixel(x, y).0;
                error += (luminance(a) - luminance(b)).abs() as f64 / 255.0;
                count += 1;
            }
        }
        count > 0 && error / (count as f64) < 0.06
    };
    let mut top = 0;
    while top < limit && row_matches(top) {
        top += 1;
    }
    let mut bottom = 0;
    while bottom < limit && row_matches(height - 1 - bottom) {
        bottom += 1;
    }
    let minimum_band = (height / 100).max(8);
    if top < minimum_band { top = 0; }
    if bottom < minimum_band { bottom = 0; }
    if top + bottom >= height / 3 {
        (0, 0)
    } else {
        (top, bottom)
    }
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
    let coarse_step = 1u32.max((max_overlap - min_overlap) / 128);
    let mut best = (min_overlap, f32::NEG_INFINITY);

    let mut runner_up = f32::NEG_INFINITY;
    let mut overlap = min_overlap;
    while overlap <= max_overlap {
        let score = overlap_score(upper, lower, overlap, x_step, max_height);
        if score > best.1 {
            if best.0.abs_diff(overlap) > (max_height / 50).max(4) {
                runner_up = best.1.max(runner_up);
            }
            best = (overlap, score);
        } else if best.0.abs_diff(overlap) > (max_height / 50).max(4) {
            runner_up = runner_up.max(score);
        }
        overlap = overlap.saturating_add(coarse_step);
        if overlap == u32::MAX { break; }
    }

    // Refine around the best coarse candidate so irregular scroll distances
    // can still land on the exact pixel offset.
    let refine_start = best.0.saturating_sub(coarse_step).max(min_overlap);
    let refine_end = best.0.saturating_add(coarse_step).min(max_overlap);
    for candidate in refine_start..=refine_end {
        let score = overlap_score(upper, lower, candidate, x_step, max_height);
        if score > best.1 {
            best = (candidate, score);
        }
    }

    // Similar-looking candidate offsets indicate repeated rows or low-texture
    // regions. Penalize ambiguous matches instead of confidently duplicating
    // content. A weak match will fail the configured confidence threshold.
    let uniqueness = if runner_up.is_finite() {
        ((best.1 - runner_up).max(0.0) / 0.08).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let confidence = best.1.clamp(0.0, 1.0) * (0.4 + 0.6 * uniqueness);
    (best.0, confidence)
}

fn overlap_score(
    upper: &RgbaImage,
    lower: &RgbaImage,
    overlap: u32,
    x_step: u32,
    max_height: u32,
) -> f32 {
    let upper_start = upper.height() - overlap;
    let mut error_sum = 0.0f64;
    let mut samples = 0u64;
    let y_step = 1u32.max(overlap / 128);
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
    (1.0 - error as f32) - (overlap as f32 / max_height as f32) * 0.015
}

fn luminance(pixel: [u8; 4]) -> f32 {
    0.2126 * f32::from(pixel[0]) + 0.7152 * f32::from(pixel[1]) + 0.0722 * f32::from(pixel[2])
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
    fn detects_common_fixed_header_and_footer() {
        let mut first = RgbaImage::new(64, 100);
        let mut second = RgbaImage::new(64, 100);
        for y in 0..100 {
            for x in 0..64 {
                let fixed = y < 10 || y >= 88;
                let value = if fixed { (x * 3 + y) as u8 } else { (x * 7 + y * 11) as u8 };
                first.put_pixel(x, y, Rgba([value, value, value, 255]));
                let second_value = if fixed { value } else { (x * 13 + y * 5) as u8 };
                second.put_pixel(x, y, Rgba([second_value, second_value, second_value, 255]));
            }
        }
        assert_eq!(detect_fixed_bars(&[first, second]), (10, 12));
    }

    #[test]
    fn exact_overlapping_rows_are_detected() {
        let mut a = RgbaImage::new(8, 20);
        let mut b = RgbaImage::new(8, 20);
        for y in 0..20 {
            for x in 0..8 {
                let color = Rgba([(x * 73 + y * 37 + x * y * 11) as u8, (x * 19 + y * y * 3 + x * y * 7) as u8, (x * x * 5 + y * 29 + x * y * 13) as u8, 255]);
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
