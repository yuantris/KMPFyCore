use core_common::CoreResult;
use core_math::Expression;
use crate::{GraphConfig, GraphPoint, GraphSegment};

pub fn sample_polar(expression: &str, config: GraphConfig, min_t: f64, max_t: f64) -> CoreResult<Vec<GraphSegment>> {
    crate::sampler::sample_polar_impl(expression, config, min_t, max_t)
}
