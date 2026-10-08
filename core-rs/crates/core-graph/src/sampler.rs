use core_math::Expr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphConfig {
    pub min_x: f64, pub max_x: f64, pub min_y: f64, pub max_y: f64,
    pub samples: usize, pub pixel_width: u32, pub pixel_height: u32,
    pub max_subdivision: u32, pub pixel_error: f64, pub max_screen_jump: f64,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self { min_x: -10.0, max_x: 10.0, min_y: -10.0, max_y: 10.0,
            samples: 1200, pixel_width: 1200, pixel_height: 800,
            max_subdivision: 12, pixel_error: 0.75, max_screen_jump: 48.0 }
    }
}

impl GraphConfig {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.min_x.is_finite() || !self.max_x.is_finite() || self.min_x >= self.max_x { return Err("invalid x range"); }
        if !self.min_y.is_finite() || !self.max_y.is_finite() || self.min_y >= self.max_y { return Err("invalid y range"); }
        if self.samples < 2 { return Err("samples must be >= 2"); }
        if self.pixel_width < 2 || self.pixel_height < 2 { return Err("pixel size must be >= 2"); }
        if self.max_subdivision == 0 { return Err("max_subdivision must be > 0"); }
        if !self.pixel_error.is_finite() || self.pixel_error <= 0.0 { return Err("pixel_error must be > 0"); }
        if !self.max_screen_jump.is_finite() || self.max_screen_jump <= 0.0 { return Err("max_screen_jump must be > 0"); }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
        if config.validate().is_err() {
            return GraphResult { segments: Vec::new(), evaluated_points: 0, discontinuities: 0 };
        }

        let mut state = State { expr, config, evaluated_points: 0 };
        let step = (config.max_x - config.min_x) / (config.samples - 1) as f64;
        let mut segments = Vec::new();
        let mut current = Vec::new();
        let mut previous = None;

        for index in 0..config.samples {
            let x = if index + 1 == config.samples { config.max_x } else { config.min_x + step * index as f64 };
            let point = state.eval(x);

            match (previous, point) {
                (None, Some(point)) => { current.push(point); previous = Some(point); }
                (Some(left), Some(right)) => {
                    let mut mids = Vec::new();
                    if state.refine(left, right, 0, &mut mids) {
                        for midpoint in mids { push_unique(&mut current, midpoint); }
                        push_unique(&mut current, right);
                        previous = Some(right);
                    } else {
                        flush(&mut segments, &mut current);
                        current.clear();
                        current.push(right);
                        previous = Some(right);
                    }
                }
                (Some(_), None) => {
                    flush(&mut segments, &mut current);
                    current.clear();
                    previous = None;
                }
                (None, None) => {}
            }
        }

        flush(&mut segments, &mut current);
        let discontinuities = segments.len().saturating_sub(1);
        GraphResult { segments, evaluated_points: state.evaluated_points, discontinuities }
    }
}

pub fn sample(expr: &Expr, config: &GraphConfig) -> GraphResult {
    GraphSampler::sample(expr, config)
}

struct State<'a> { expr: &'a Expr, config: &'a GraphConfig, evaluated_points: usize }

impl State<'_> {
    fn eval(&mut self, x: f64) -> Option<GraphPoint> {
        self.evaluated_points += 1;
        let y = self.expr.eval_x(x);
        y.is_finite().then_some(GraphPoint { x, y })
    }

    fn refine(&mut self, left: GraphPoint, right: GraphPoint, depth: u32, mids: &mut Vec<GraphPoint>) -> bool {
        let midpoint_x = left.x + (right.x - left.x) * 0.5;
        if midpoint_x == left.x || midpoint_x == right.x { return self.safe(left, right); }

        let Some(midpoint) = self.eval(midpoint_x) else { return false; };
        let left_y = self.screen_y(left);
        let mid_y = self.screen_y(midpoint);
        let right_y = self.screen_y(right);
        let midpoint_error = (mid_y - (left_y + right_y) * 0.5).abs();
        let screen_jump = (right_y - left_y).abs();

        if midpoint_error <= self.config.pixel_error && screen_jump <= self.config.max_screen_jump { return true; }
        if depth >= self.config.max_subdivision { return false; }

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
        (self.config.max_y - point.y) / (self.config.max_y - self.config.min_y) * (self.config.pixel_height - 1) as f64
    }
}

fn push_unique(points: &mut Vec<GraphPoint>, point: GraphPoint) {
    if points.last().is_some_and(|last| last.x == point.x) { return; }
    points.push(point);
}

fn flush(segments: &mut Vec<GraphSegment>, points: &mut Vec<GraphPoint>) {
    if points.len() >= 2 { segments.push(GraphSegment { points: std::mem::take(points) }); }
    else { points.clear(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::Expression;

    fn config() -> GraphConfig {
        GraphConfig { min_x: -4.0, max_x: 4.0, min_y: -20.0, max_y: 20.0,
            samples: 101, pixel_width: 800, pixel_height: 600,
            max_subdivision: 14, pixel_error: 0.75, max_screen_jump: 48.0 }
    }

    #[test]
    fn reciprocal_splits() {
        let result = sample(&Expression::compile("1/x").unwrap(), &config());
        assert!(result.segments.len() >= 2);
        for segment in &result.segments {
            assert!(!(segment.points.first().unwrap().x < 0.0 && segment.points.last().unwrap().x > 0.0));
        }
    }

    #[test]
    fn polynomial_stays_connected() {
        let result = sample(&Expression::compile("x^2").unwrap(), &config());
        assert_eq!(result.segments.len(), 1);
    }

    #[test]
    fn domain_splits() {
        let result = sample(&Expression::compile("sqrt(x)").unwrap(), &config());
        assert!(!result.segments.is_empty());
        assert!(result.segments[0].points[0].x >= 0.0);
    }

    #[test]
    fn sine_stays_connected() {
        let result = sample(&Expression::compile("sin(x)").unwrap(), &config());
        assert_eq!(result.segments.len(), 1);
    }

    #[test]
    fn tangent_splits() {
        let result = sample(&Expression::compile("tan(x)").unwrap(), &config());
        assert!(result.segments.len() > 1);
    }
}
