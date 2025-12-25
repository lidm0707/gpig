use gpui::{Pixels, Point};

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: Point<Pixels>,
    pub to: Point<Pixels>,
}

impl Edge {
    pub fn new(x: Pixels, y: Pixels) -> Self {
        Self {
            from: Point::new(x, y),
            to: Point::new(0.0.into(), 0.0.into()),
        }
    }
}
#[derive(Debug, Clone, Default)]
pub struct EdgeManager {
    pub edges: Vec<Edge>,
}

impl EdgeManager {
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    pub fn add(&mut self, from: Point<Pixels>, to: Point<Pixels>) {
        self.edges.push(Edge { from, to });
    }
}
