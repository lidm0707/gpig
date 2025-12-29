use git2::{Oid, Time};
use gpui::{Pixels, Point};
#[derive(Debug, Clone)]
pub struct CommitNode {
    pub oid: Oid,
    pub message: String,
    pub author: String,
    pub timestamp: Time,
    pub parents: Vec<Oid>,
    pub position: Point<Pixels>,
    pub color: usize,
}

impl CommitNode {
    pub fn new(
        oid: Oid,
        message: String,
        author: String,
        timestamp: Time,
        parents: Vec<Oid>,
        position: Point<Pixels>,
        color: usize,
    ) -> Self {
        CommitNode {
            oid,
            message,
            author,
            timestamp,
            parents,
            position,
            color,
        }
    }
}
