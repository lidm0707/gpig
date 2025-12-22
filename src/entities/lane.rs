use git2::Oid;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LaneManager {
    pub lanes: Vec<Option<Oid>>,
}

impl LaneManager {
    pub fn new() -> Self {
        LaneManager { lanes: Vec::new() }
    }

    pub fn get_lanes(&self) -> &[Option<Oid>] {
        &self.lanes
    }

    pub fn assign_commit(&mut self, commit_oid: &Oid, parent_oids: &[Oid]) -> usize {
        if self.lanes.is_empty() {
            self.lanes
                .splice(0..0, parent_oids.iter().map(|oid| Some(*oid)));
            return 1;
        }

        if self.lanes.contains(&Some(*commit_oid)) {
            let index = self
                .lanes
                .iter()
                .position(|oid| oid == &Some(*commit_oid))
                .unwrap();
            self.lanes[index] = None;

            for parent_oid in parent_oids {
                for i in 0..self.lanes.len() {
                    if self.lanes[i].is_none() {
                        self.lanes[i] = Some(*parent_oid);
                        break;
                    } else {
                        self.lanes.push(Some(*parent_oid));
                    }
                }
            }

            return index;
        } else {
            for parent_oid in parent_oids {
                for i in 0..self.lanes.len() {
                    if self.lanes[i].is_none() {
                        self.lanes[i] = Some(*parent_oid);
                        break;
                    } else {
                        self.lanes.push(Some(*parent_oid));
                    }
                }
            }
            let mut count = 0;
            for node in self.lanes.iter() {
                if !node.is_some() {
                    count += 1;
                }
            }

            return count;
        }
    }
}
