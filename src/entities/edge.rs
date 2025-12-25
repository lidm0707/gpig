use gpui::{Pixels, Point};

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: Point<Pixels>,
    pub to: Point<Pixels>,
}

impl Edge {
    pub fn new(from: Point<Pixels>, to: Point<Pixels>) -> Self {
        Self { from, to }
    }
}

#[derive(Debug, Default)]
pub struct EdgeManager {
    edges: Vec<Edge>,
}

impl EdgeManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, from: Point<Pixels>, to: Point<Pixels>) {
        self.edges.push(Edge::new(from, to));
    }

    pub fn take_edges(&mut self) -> Vec<Edge> {
        std::mem::take(&mut self.edges)
    }
}
