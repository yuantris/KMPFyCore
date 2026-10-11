use thiserror::Error;

#[derive(Debug, Error)]
pub enum StitchError {
    #[error("at least one input image is required")]
    EmptyInput,
    #[error("invalid stitching configuration: {0}")]
    InvalidConfig(String),
    #[error("input image processing failed: {0}")]
    Image(String),
    #[error("input images have invalid dimensions")]
    InvalidDimensions,
    #[error("no reliable overlap match found between images {left} and {right} (confidence {confidence:.3})")]
    MatchFailed { left: usize, right: usize, confidence: f32 },
    #[error("output exceeds the configured size or pixel budget")]
    OutputTooLarge,
    #[error("panorama mode requires the optional OpenCV backend, which is not enabled")]
    OpenCvUnavailable,
}
