use core_math::Expression;
use crate::{GraphConfig, GraphPoint, GraphResult, GraphSampler, GraphSegment};

pub fn sample_cartesian(expression: &str, config: GraphConfig) -> Result<GraphResult, core_common::CoreError> {
    GraphSampler::new(config).sample_checked(expression)
}

pub(crate) fn evaluate(expression: &core_math::Expr, x: f64) -> f64 {
    expression.eval_x(x)
}

pub(crate) fn point(x: f64, y: f64) -> GraphPoint { GraphPoint { x, y } }

pub(crate) fn segment(points: Vec<GraphPoint>) -> GraphSegment { GraphSegment { points } }
