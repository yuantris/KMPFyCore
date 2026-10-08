use core_math::Expr;
use core_common::{CoreError, CoreResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphConfig {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub samples: usize,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub max_subdivision: u32,
    pub pixel_error: f64,
    pub max_screen_jump: f64,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            min_x: -10.0, max_x: 10.0, min_y: -10.0, max_y: 10.0,
            samples: 1200, pixel_width: 1200, pixel_height: 800,
            max_subdivision: 12, pixel_error: 0.75, max_screen_jump: 48.0,
        }
    }
}

impl GraphConfig {
    pub fn validate(&self) -> CoreResult<()> {
        if !self.min_x.is_finite() || !self.max_x.is_finite() || self.min_x >= self.max_x {
            return Err(CoreError::InvalidArgument("invalid x range".into()));
        }
        if !self.min_y.is_finite() || !self.max_y.is_finite() || self.min_y >= self.max_y {
            return Err(CoreError::InvalidArgument("invalid y range".into()));
        }
        if !(2..=1_000_000).contains(&self.samples) {
            return Err(CoreError::InvalidArgument("samples must be in 2..=1000000".into()));
        }
        if !(2..=16_384).contains(&self.pixel_width) || !(2..=16_384).contains(&self.pixel_height) {
            return Err(CoreError::InvalidArgument("pixel size must be in 2..=16384".into()));
        }
        if self.max_subdivision == 0 || self.max_subdivision > 24 {
            return Err(CoreError::InvalidArgument("max_subdivision must be in 1..=24".into()));
        }
        if !self.pixel_error.is_finite() || self.pixel_error <= 0.0 {
            return Err(CoreError::InvalidArgument("pixel_error must be > 0".into()));
        }
        if !self.max_screen_jump.is_finite() || self.max_screen_jump <= 0.0 {
            return Err(CoreError::InvalidArgument("max_screen_jump must be > 0".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct GraphPoint { pub x: f64, pub y: f64 }

#[derive(Debug, Clone, PartialEq)]
pub struct GraphSegment { pub points: Vec<GraphPoint> }

#[derive(Debug, Clone, PartialEq)]
pub struct GraphResult {
    pub segments: Vec<GraphSegment>,
    pub evaluated_points: usize,
    pub discontinuities: usize,
}

pub struct GraphSampler;

impl GraphSampler {
    pub fn sample(expr: &Expr, config: &GraphConfig) -> GraphResult {
        Self::sample_checked(expr, config).unwrap_or(GraphResult {
            segments: Vec::new(), evaluated_points: 0, discontinuities: 0,
        })
    }

    pub fn sample_checked(expr: &Expr, config: &GraphConfig) -> CoreResult<GraphResult> {
        config.validate()?;

        let mut state = State { expr, config, evaluated_points: 0, discontinuities: 0 };
        let step = (config.max_x - config.min_x) / (config.samples - 1) as f64;
        let mut segments = Vec::new();
        let mut current = Vec::new();
        let mut previous = None;

        for index in 0..config.samples {
            let x = if index + 1 == config.samples {
                config.max_x
            } else {
                config.min_x + step * index as f64
            };

            let point = state.eval(x);
            match (previous, point) {
                (None, Some(point)) => {
                    current.push(point);
                    previous = Some(point);
                }
                (Some(left), Some(right)) => {
                    let mut mids = Vec::new();
                    if state.refine(left, right, 0, &mut mids) {
                        for midpoint in mids { push_unique(&mut current, midpoint); }
                        push_unique(&mut current, right);
                        previous = Some(right);
                    } else {
                        flush(&mut segments, &mut current);
                        state.discontinuities += 1;
                        current.clear();
                        current.push(right);
                        previous = Some(right);
                    }
                }
                (Some(_), None) => {
                    flush(&mut segments, &mut current);
                    state.discontinuities += 1;
                    current.clear();
                    previous = None;
                }
                (None, None) => {}
            }
        }

        flush(&mut segments, &mut current);
        Ok(GraphResult {
            segments,
            evaluated_points: state.evaluated_points,
            discontinuities: state.discontinuities,
        })
    }
}

pub fn sample(expr: &Expr, config: &GraphConfig) -> GraphResult {
    GraphSampler::sample(expr, config)
}

pub fn sample_checked(expr: &Expr, config: &GraphConfig) -> CoreResult<GraphResult> {
    GraphSampler::sample_checked(expr, config)
}

struct State<'a> {
    expr: &'a Expr,
    config: &'a GraphConfig,
    evaluated_points: usize,
    discontinuities: usize,
}

impl State<'_> {
    fn eval(&mut self, x: f64) -> Option<GraphPoint> {
        self.evaluated_points += 1;
        let y = self.expr.eval_x(x);
        y.is_finite().then_some(GraphPoint { x, y })
    }

    fn refine(&mut self, left: GraphPoint, right: GraphPoint, depth: u32, mids: &mut Vec<GraphPoint>) -> bool {
        let midpoint_x = left.x + (right.x - left.x) * 0.5;
        if midpoint_x == left.x || midpoint_x == right.x {
            return self.safe(left, right);
        }

        let Some(midpoint) = self.eval(midpoint_x) else { return false; };
        let left_y = self.screen_y(left);
        let mid_y = self.screen_y(midpoint);
        let right_y = self.screen_y(right);
        let midpoint_error = (mid_y - (left_y + right_y) * 0.5).abs();
        let screen_jump = (right_y - left_y).abs();

        if midpoint_error <= self.config.pixel_error && screen_jump <= self.config.max_screen_jump {
            return true;
        }
        if depth >= self.config.max_subdivision {
            return midpoint_error <= self.config.pixel_error;
        }

        let mut left_mids = Vec::new();
        if !self.refine(left, midpoint, depth + 1, &mut left_mids) { return false; }
        let mut right_mids = Vec::new();
        if !self.refine(midpoint, right, depth + 1, &mut right_mids) { return false; }

        mids.extend(left_mids);
        mids.push(midpoint);
        mids.extend(right_mids);
        true
    }

    fn safe(&self, left: GraphPoint, right: GraphPoint) -> bool {
        (self.screen_y(right) - self.screen_y(left)).abs() <= self.config.max_screen_jump
    }

    fn screen_y(&self, point: GraphPoint) -> f64 {
        (self.config.max_y - point.y)
            / (self.config.max_y - self.config.min_y)
            * (self.config.pixel_height - 1) as f64
    }
}

fn push_unique(points: &mut Vec<GraphPoint>, point: GraphPoint) {
    if points.last().is_some_and(|last| last.x == point.x) { return; }
    points.push(point);
}

fn flush(segments: &mut Vec<GraphSegment>, points: &mut Vec<GraphPoint>) {
    if points.len() >= 2 {
        segments.push(GraphSegment { points: std::mem::take(points) });
    } else {
        points.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::Expression;

    fn config() -> GraphConfig {
        GraphConfig {
            min_x: -4.0, max_x: 4.0, min_y: -20.0, max_y: 20.0,
            samples: 101, pixel_width: 800, pixel_height: 600,
            max_subdivision: 14, pixel_error: 0.75, max_screen_jump: 48.0,
        }
    }

    #[test]
    fn reciprocal_splits() {
        let result = sample_checked(&Expression::compile("1/x").unwrap(), &config()).unwrap();
        assert!(result.segments.len() >= 2);
    }

    #[test]
    fn polynomial_stays_connected() {
        let result = sample_checked(&Expression::compile("x^2").unwrap(), &config()).unwrap();
        assert_eq!(result.segments.len(), 1);
    }

    #[test]
    fn domain_is_split() {
        let result = sample_checked(&Expression::compile("sqrt(x)").unwrap(), &config()).unwrap();
        assert!(!result.segments.is_empty());
        assert!(result.segments[0].points[0].x >= 0.0);
    }

    #[test]
    fn sine_stays_connected() {
        let result = sample_checked(&Expression::compile("sin(x)").unwrap(), &config()).unwrap();
        assert_eq!(result.segments.len(), 1);
    }

    #[test]
    fn tangent_splits() {
        let result = sample_checked(&Expression::compile("tan(x)").unwrap(), &config()).unwrap();
        assert!(result.segments.len() > 1);
    }

    #[test]
    fn invalid_config_is_reported() {
        let mut config = config();
        config.samples = 1;
        assert!(sample_checked(&Expression::compile("x").unwrap(), &config()).is_err());
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphMode {
    Cartesian,
    Polar,
    Parametric,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParametricConfig {
    pub min_t: f64,
    pub max_t: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct GraphAnalysis {
    pub zeroes: Vec<GraphPoint>,
    pub extrema: Vec<GraphPoint>,
}

pub fn sample_polar(expr: &Expr, config: &GraphConfig, min_t: f64, max_t: f64) -> CoreResult<GraphResult> {
    sample_parametric_pair(expr, &Expr::Number(0.0), config, min_t, max_t, true)
}

pub fn sample_polar_expression(expression: &str, config: &GraphConfig, min_t: f64, max_t: f64) -> CoreResult<GraphResult> {
    let expr = core_math::Expression::compile(expression)?;
    sample_polar(&expr, config, min_t, max_t)
}

pub fn sample_parametric_expressions(x_expression: &str, y_expression: &str, config: &GraphConfig, min_t: f64, max_t: f64) -> CoreResult<GraphResult> {
    let x_expr = core_math::Expression::compile(x_expression)?;
    let y_expr = core_math::Expression::compile(y_expression)?;
    sample_parametric(&x_expr, &y_expr, config, min_t, max_t)
}

pub fn sample_parametric(x_expr: &Expr, y_expr: &Expr, config: &GraphConfig, min_t: f64, max_t: f64) -> CoreResult<GraphResult> {
    sample_parametric_pair(x_expr, y_expr, config, min_t, max_t, false)
}

fn sample_parametric_pair(
    first: &Expr,
    second: &Expr,
    config: &GraphConfig,
    min_t: f64,
    max_t: f64,
    polar: bool,
) -> CoreResult<GraphResult> {
    config.validate()?;
    if !min_t.is_finite() || !max_t.is_finite() || min_t >= max_t {
        return Err(CoreError::InvalidArgument("invalid parameter range".into()));
    }

    let base_count = config.samples.max(64).min(20_000);
    let step = (max_t - min_t) / (base_count - 1) as f64;
    let mut state = ParamState {
        first,
        second,
        config,
        polar,
        evaluated_points: 0,
    };
    let mut segments = Vec::new();
    let mut current = Vec::new();

    for i in 0..base_count - 1 {
        let t0 = min_t + step * i as f64;
        let t1 = if i + 2 == base_count { max_t } else { t0 + step };
        let p0 = state.eval(t0);
        let p1 = state.eval(t1);

        match (p0, p1) {
            (Some(a), Some(b)) => {
                let mut mids = Vec::new();
                if state.refine(a, b, t0, t1, 0, &mut mids) {
                    push_unique(&mut current, a);
                    mids.into_iter().for_each(|p| push_unique(&mut current, p));
                    push_unique(&mut current, b);
                } else {
                    flush(&mut segments, &mut current);
                }
            }
            _ => flush(&mut segments, &mut current),
        }
    }
    flush(&mut segments, &mut current);

    Ok(GraphResult {
        segments,
        evaluated_points: state.evaluated_points,
        discontinuities: 0,
    })
}

struct ParamState<'a> {
    first: &'a Expr,
    second: &'a Expr,
    config: &'a GraphConfig,
    polar: bool,
    evaluated_points: usize,
}

impl ParamState<'_> {
    fn eval(&mut self, t: f64) -> Option<GraphPoint> {
        self.evaluated_points += 1;
        let (x, y) = if self.polar {
            let r = self.first.eval_vars(t, 0.0, t);
            (r * t.cos(), r * t.sin())
        } else {
            (
                self.first.eval_vars(0.0, 0.0, t),
                self.second.eval_vars(0.0, 0.0, t),
            )
        };
        (x.is_finite() && y.is_finite()).then_some(GraphPoint { x, y })
    }

    fn screen(&self, p: GraphPoint) -> (f64, f64) {
        (
            (p.x - self.config.min_x) / (self.config.max_x - self.config.min_x)
                * self.config.pixel_width as f64,
            (self.config.max_y - p.y) / (self.config.max_y - self.config.min_y)
                * self.config.pixel_height as f64,
        )
    }

    fn refine(
        &mut self,
        left: GraphPoint,
        right: GraphPoint,
        left_t: f64,
        right_t: f64,
        depth: u32,
        mids: &mut Vec<GraphPoint>,
    ) -> bool {
        let mid_t = (left_t + right_t) * 0.5;
        if mid_t == left_t || mid_t == right_t {
            return self.safe(left, right);
        }
        let Some(mid) = self.eval(mid_t) else { return false; };
        let (lx, ly) = self.screen(left);
        let (mx, my) = self.screen(mid);
        let (rx, ry) = self.screen(right);
        let line_x = (lx + rx) * 0.5;
        let line_y = (ly + ry) * 0.5;
        let error = ((mx - line_x).powi(2) + (my - line_y).powi(2)).sqrt();
        let jump = ((rx - lx).powi(2) + (ry - ly).powi(2)).sqrt();

        if error <= self.config.pixel_error && jump <= self.config.max_screen_jump {
            return true;
        }
        if depth >= self.config.max_subdivision {
            return error <= self.config.pixel_error * 2.0 && jump <= self.config.max_screen_jump * 2.0;
        }

        let mut left_mids = Vec::new();
        if !self.refine(left, mid, left_t, mid_t, depth + 1, &mut left_mids) { return false; }
        let mut right_mids = Vec::new();
        if !self.refine(mid, right, mid_t, right_t, depth + 1, &mut right_mids) { return false; }
        mids.extend(left_mids);
        mids.push(mid);
        mids.extend(right_mids);
        true
    }

    fn safe(&self, left: GraphPoint, right: GraphPoint) -> bool {
        let (lx, ly) = self.screen(left);
        let (rx, ry) = self.screen(right);
        ((rx - lx).powi(2) + (ry - ly).powi(2)).sqrt() <= self.config.max_screen_jump
    }
}

pub fn analyze(expr: &Expr, config: &GraphConfig) -> CoreResult<GraphAnalysis> {
    config.validate()?;
    let n = config.samples.clamp(256, 100_000);
    let step = (config.max_x - config.min_x) / (n - 1) as f64;
    let mut values: Vec<(f64, f64)> = Vec::with_capacity(n);
    for i in 0..n {
        let x = if i + 1 == n { config.max_x } else { config.min_x + step * i as f64 };
        let y = expr.eval_x(x);
        if y.is_finite() { values.push((x, y)); }
    }

    let mut zeroes = Vec::new();
    let mut extrema = Vec::new();
    for pair in values.windows(2) {
        let (x1, y1) = pair[0];
        let (x2, y2) = pair[1];
        if y1 == 0.0 {
            zeroes.push(GraphPoint { x: x1, y: 0.0 });
        } else if y1.signum() != y2.signum() {
            let mut lo = x1;
            let mut hi = x2;
            for _ in 0..48 {
                let mid = (lo + hi) * 0.5;
                let ym = expr.eval_x(mid);
                if !ym.is_finite() { break; }
                if y1.signum() == ym.signum() { lo = mid; } else { hi = mid; }
            }
            let root = (lo + hi) * 0.5;
            zeroes.push(GraphPoint { x: root, y: 0.0 });
        }
    }
    for pair in values.windows(3) {
        let (x0, y0) = pair[0];
        let (x1, y1) = pair[1];
        let (x2, y2) = pair[2];
        let d1 = y1 - y0;
        let d2 = y2 - y1;
        if d1.signum() != 0.0 && d2.signum() != 0.0 && d1.signum() != d2.signum() && y1.is_finite() {
            extrema.push(GraphPoint { x: x1, y: y1 });
        }
        let _ = (x0, x2);
    }
    zeroes.sort_by(|a,b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    zeroes.dedup_by(|a,b| (a.x-b.x).abs() < step * 2.0);
    extrema.sort_by(|a,b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    extrema.dedup_by(|a,b| (a.x-b.x).abs() < step * 2.0);
    Ok(GraphAnalysis { zeroes, extrema })
}
