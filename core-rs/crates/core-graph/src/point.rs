use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct GraphPoint {
    pub x: f64,
    pub y: f64,
}

impl GraphPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphSegment {
    pub points: Vec<GraphPoint>,
}

impl GraphSegment {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
        }
    }

    pub fn push(&mut self, point: GraphPoint) {
        self.points.push(point);
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphResult {
    pub segments: Vec<GraphSegment>,
}

impl GraphResult {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }
}