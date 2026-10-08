use core_common::CoreResult;
use crate::{GraphConfig, GraphSegment};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParametricConfig {
    pub min_t: f64,
    pub max_t: f64,
}

pub fn sample_parametric(
    x_expression: &str,
    y_expression: &str,
    config: GraphConfig,
    parametric: ParametricConfig,
) -> CoreResult<Vec<GraphSegment>> {
    crate::sampler::sample_parametric_impl(x_expression, y_expression, config, parametric)
}
