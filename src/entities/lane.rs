use git2::Oid;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Lane {
    pub id: usize,
    pub commits: Vec<Oid>,
    pub color: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LaneManager {
    pub lanes: Vec<Lane>,
    pub commit_lanes: HashMap<Oid, usize>, // Maps commit ID to lane ID
}

impl Lane {
    pub fn new(id: usize) -> Self {
        Lane {
            id,
            commits: Vec::new(),
            color: None,
        }
    }

    pub fn with_color(id: usize, color: String) -> Self {
        Lane {
            id,
            commits: Vec::new(),
            color: Some(color),
        }
    }

    pub fn add_commit(&mut self, commit_oid: Oid) {
        self.commits.push(commit_oid);
    }
}

impl LaneManager {
    pub fn new() -> Self {
        LaneManager {
            lanes: Vec::new(),
            commit_lanes: HashMap::new(),
        }
    }

    pub fn assign_commit(&mut self, commit_oid: Oid, parent_oids: &[Oid]) -> usize {
        // Check if commit is already in a lane
        if let Some(&lane_id) = self.commit_lanes.get(&commit_oid) {
            return lane_id;
        }

        // Determine lane based on parents
        if parent_oids.is_empty() {
            // Initial commit - create first lane
            self.create_new_lane(commit_oid)
        } else if parent_oids.len() == 1 {
            // Single parent - check if parent is already in a lane
            if let Some(&parent_lane_id) = self.commit_lanes.get(&parent_oids[0]) {
                // Use parent's lane
                self.lanes[parent_lane_id].add_commit(commit_oid);
                self.commit_lanes.insert(commit_oid, parent_lane_id);
                parent_lane_id
            } else {
                // Parent not processed yet, create a new lane
                self.create_new_lane(commit_oid)
            }
        } else {
            // Multiple parents - merge situation
            // Create a new lane for the merge commit
            self.create_new_lane(commit_oid)
        }
    }

    fn create_new_lane(&mut self, commit_oid: Oid) -> usize {
        let lane_id = self.lanes.len();
        let mut lane = Lane::new(lane_id);
        lane.add_commit(commit_oid);
        self.lanes.push(lane);
        self.commit_lanes.insert(commit_oid, lane_id);
        lane_id
    }

    pub fn get_lane_position(&self, commit_oid: Oid) -> Option<usize> {
        self.commit_lanes.get(&commit_oid).copied()
    }
}
