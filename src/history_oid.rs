use std::collections::HashMap;

use git2::Oid;
use gpui::{Pixels, Point};

pub struct HistoryOid {
    pub point: Point<Pixels>,
    pub color: usize,
    pub lane: usize,
}

impl HistoryOid {
    pub fn new(point: Point<Pixels>, color: usize, lane: usize) -> Self {
        Self { point, color, lane }
    }
}

pub struct HistoryOids {
    pub history_oid: HashMap<Oid, Vec<HistoryOid>>,
}

impl HistoryOids {
    pub fn new() -> Self {
        Self {
            history_oid: HashMap::new(),
        }
    }
    pub fn add_history(&mut self, oid: Oid, history_oid: HistoryOid) {
        self.history_oid
            .entry(oid)
            .or_insert_with(Vec::new)
            .push(history_oid);
    }
    pub fn get(&self, oid: &Oid) -> Option<&Vec<HistoryOid>> {
        self.history_oid.get(oid)
    }
}
