use super::Expr;

#[derive(Debug, Clone, Copy)]
pub struct GraphPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct GraphSegment {
    pub points: Vec<GraphPoint>,
}

#[derive(Debug, Clone)]
pub struct GraphConfig {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub samples: usize,
    pub jump_factor: f64,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self { min_x: -10.0, max_x: 10.0, min_y: -10.0, max_y: 10.0, samples: 1200, jump_factor: 24.0 }
    }
}

pub fn sample(expr: &Expr, config: &GraphConfig) -> Vec<GraphSegment> {
    let count = config.samples.max(2);
    let dx = (config.max_x - config.min_x) / (count - 1) as f64;
    let y_span = (config.max_y - config.min_y).abs().max(1e-9);
    let jump_limit = y_span * config.jump_factor.max(1.0);

    let mut segments = Vec::new();
    let mut current = Vec::new();

    for i in 0..count {
        let x = config.min_x + dx * i as f64;
        let y = expr.eval_x(x);

        let valid = y.is_finite()
            && y.abs() <= config.max_y.abs().max(config.min_y.abs()).max(1.0) * 1000.0;

        if !valid {
            flush(&mut current, &mut segments);
            continue;
        }

        if let Some(previous) = current.last() {
            if (y - previous.y).abs() > jump_limit {
                flush(&mut current, &mut segments);
            }
        }

        current.push(GraphPoint { x, y });
    }

    flush(&mut current, &mut segments);
    segments
}

fn flush(current: &mut Vec<GraphPoint>, segments: &mut Vec<GraphSegment>) {
    if current.len() >= 2 {
        segments.push(GraphSegment { points: std::mem::take(current) });
    } else {
        current.clear();
    }
}
