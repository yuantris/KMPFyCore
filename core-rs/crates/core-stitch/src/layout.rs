use image::{imageops, Rgba, RgbaImage};

use crate::{
    config::{StitchConfig, StitchMode},
    error::StitchError,
    StitchMetadata, StitchResult, StitchSegment,
};

pub fn stitch_layout(
    images: &[RgbaImage],
    config: &StitchConfig,
) -> Result<StitchResult, StitchError> {
    if images.is_empty() {
        return Err(StitchError::EmptyInput);
    }
    config.validate()?;

    match config.mode {
        StitchMode::Vertical => stitch_linear(images, config, false),
        StitchMode::Horizontal => stitch_linear(images, config, true),
        StitchMode::Grid => stitch_grid(images, config),
        _ => Err(StitchError::InvalidConfig(
            "layout engine only accepts vertical, horizontal, or grid mode".into(),
        )),
    }
}

fn checked_canvas(width: u64, height: u64, config: &StitchConfig) -> Result<RgbaImage, StitchError> {
    if width == 0 || height == 0 || width > u32::MAX as u64 || height > u32::MAX as u64
        || width.checked_mul(height).is_none_or(|pixels| pixels > config.max_output_pixels)
    {
        return Err(StitchError::OutputTooLarge);
    }
    Ok(RgbaImage::from_pixel(
        width as u32,
        height as u32,
        Rgba(config.background),
    ))
}

fn stitch_linear(
    images: &[RgbaImage],
    config: &StitchConfig,
    horizontal: bool,
) -> Result<StitchResult, StitchError> {
    let target = if horizontal {
        images.iter().map(RgbaImage::height).max()
    } else {
        images.iter().map(RgbaImage::width).max()
    }.ok_or(StitchError::EmptyInput)?;

    let mut normalized = Vec::with_capacity(images.len());
    for image in images {
        if image.width() == 0 || image.height() == 0 {
            return Err(StitchError::InvalidDimensions);
        }
        let (width, height) = if horizontal {
            let width = (u64::from(image.width()) * u64::from(target) / u64::from(image.height()))
                .max(1);
            (u32::try_from(width).map_err(|_| StitchError::OutputTooLarge)?, target)
        } else {
            let height = (u64::from(image.height()) * u64::from(target) / u64::from(image.width()))
                .max(1);
            (target, u32::try_from(height).map_err(|_| StitchError::OutputTooLarge)?)
        };
        normalized.push(imageops::resize(image, width, height, imageops::FilterType::Lanczos3));
    }

    let gaps = u64::from(config.spacing) * images.len().saturating_sub(1) as u64;
    let (width, height) = if horizontal {
        (normalized.iter().map(|i| u64::from(i.width())).sum::<u64>() + gaps, u64::from(target))
    } else {
        (u64::from(target), normalized.iter().map(|i| u64::from(i.height())).sum::<u64>() + gaps)
    };
    let (mut canvas, scale) = bounded_canvas(width, height, config)?;
    let mut offset = 0u64;
    let mut segments = Vec::with_capacity(images.len());

    for (index, image) in normalized.iter().enumerate() {
        let (x, y) = if horizontal { (offset, 0) } else { (0, offset) };
        imageops::overlay(&mut canvas, image, x as i64, y as i64);
        segments.push(StitchSegment {
            image_index: index,
            x: (x as f64 * scale) as i64,
            y: (y as f64 * scale) as i64,
            crop_x: 0,
            crop_y: 0,
            crop_width: image.width(),
            crop_height: image.height(),
            confidence: 1.0,
        });
        offset += if horizontal { u64::from(image.width()) } else { u64::from(image.height()) };
        if index + 1 < normalized.len() {
            offset += u64::from(config.spacing);
        }
    }

    Ok(StitchResult {
        metadata: StitchMetadata {
            width: canvas.width(),
            height: canvas.height(),
            image_count: images.len(),
            mode: config.mode,
            segments,
        },
        image: canvas,
    })
}

fn stitch_grid(images: &[RgbaImage], config: &StitchConfig) -> Result<StitchResult, StitchError> {
    let cell_width = images.iter().map(RgbaImage::width).max().ok_or(StitchError::EmptyInput)?;
    let cell_height = images.iter().map(RgbaImage::height).max().ok_or(StitchError::EmptyInput)?;
    if images.iter().any(|i| i.width() == 0 || i.height() == 0) {
        return Err(StitchError::InvalidDimensions);
    }

    let columns = if config.columns == 0 {
        (images.len() as f64).sqrt().ceil() as usize
    } else {
        config.columns as usize
    }.clamp(1, images.len());
    let rows = images.len().div_ceil(columns);
    let gap = u64::from(config.spacing);
    let width = u64::from(cell_width) * columns as u64 + gap * columns.saturating_sub(1) as u64;
    let height = u64::from(cell_height) * rows as u64 + gap * rows.saturating_sub(1) as u64;
    let (mut canvas, scale) = bounded_canvas(width, height, config)?;
    let mut segments = Vec::with_capacity(images.len());

    for (index, image) in images.iter().enumerate() {
        let col = index % columns;
        let row = index / columns;
        let x = col as u64 * (u64::from(cell_width) + gap);
        let y = row as u64 * (u64::from(cell_height) + gap);
        imageops::overlay(&mut canvas, image, x as i64, y as i64);
        segments.push(StitchSegment {
            image_index: index,
            x: (x as f64 * scale) as i64,
            y: (y as f64 * scale) as i64,
            crop_x: 0,
            crop_y: 0,
            crop_width: image.width(),
            crop_height: image.height(),
            confidence: 1.0,
        });
    }

    Ok(StitchResult {
        metadata: StitchMetadata {
            width: canvas.width(),
            height: canvas.height(),
            image_count: images.len(),
            mode: StitchMode::Grid,
            segments,
        },
        image: canvas,
    })
}

/// Compute a safe canvas and uniformly downscale if explicit output dimensions require it.
fn bounded_canvas(
    width: u64,
    height: u64,
    config: &StitchConfig,
) -> Result<(RgbaImage, f64), StitchError> {
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
    let mut scaled_width = ((width as f64 * scale).floor() as u64).max(1);
    let mut scaled_height = ((height as f64 * scale).floor() as u64).max(1);
    let pixels = scaled_width.checked_mul(scaled_height).ok_or(StitchError::OutputTooLarge)?;
    if pixels > config.max_output_pixels {
        let pixel_scale = (config.max_output_pixels as f64 / pixels as f64).sqrt();
        scale *= pixel_scale;
        scaled_width = ((width as f64 * scale).floor() as u64).max(1);
        scaled_height = ((height as f64 * scale).floor() as u64).max(1);
    }
    if scaled_width > u32::MAX as u64 || scaled_height > u32::MAX as u64
        || scaled_width.checked_mul(scaled_height).is_none_or(|p| p > config.max_output_pixels)
    {
        return Err(StitchError::OutputTooLarge);
    }

    let original = checked_canvas(width.min(u32::MAX as u64), height.min(u32::MAX as u64), config);
    // Avoid allocating the unscaled image when the final dimensions are smaller.
    let _ = original;
    Ok((RgbaImage::from_pixel(
        scaled_width as u32,
        scaled_height as u32,
        Rgba(config.background),
    ), scale))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(mode: StitchMode) -> StitchConfig {
        StitchConfig { mode, spacing: 0, ..StitchConfig::default() }
    }

    #[test]
    fn vertical_layout_stacks_images() {
        let a = RgbaImage::from_pixel(2, 3, Rgba([255, 0, 0, 255]));
        let b = RgbaImage::from_pixel(2, 4, Rgba([0, 255, 0, 255]));
        let result = stitch_layout(&[a, b], &config(StitchMode::Vertical)).expect("stitch");
        assert_eq!((result.image.width(), result.image.height()), (2, 7));
        assert_eq!(result.metadata.segments[1].y, 3);
    }

    #[test]
    fn grid_places_images_in_cells() {
        let a = RgbaImage::from_pixel(2, 2, Rgba([255, 0, 0, 255]));
        let b = RgbaImage::from_pixel(2, 2, Rgba([0, 255, 0, 255]));
        let cfg = StitchConfig { mode: StitchMode::Grid, columns: 2, spacing: 1, ..StitchConfig::default() };
        let result = stitch_layout(&[a, b], &cfg).expect("stitch");
        assert_eq!((result.image.width(), result.image.height()), (5, 2));
        assert_eq!(result.metadata.segments[1].x, 3);
    }
}
