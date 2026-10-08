mod analysis;
mod cartesian;
mod parametric;
mod polar;
mod sampler;

pub use analysis::analyze;
pub use cartesian::sample_cartesian;
pub use parametric::{sample_parametric, ParametricConfig};
pub use polar::sample_polar;
pub use sampler::{sample, sample_checked, GraphConfig, GraphMode, GraphPoint, GraphResult, GraphSampler, GraphSegment};
