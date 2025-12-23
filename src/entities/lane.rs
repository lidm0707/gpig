use git2::Oid;

#[derive(Debug, Clone)]
pub struct LaneManager {
    pub lanes: Vec<Option<Oid>>,
}

impl LaneManager {
    pub fn new() -> Self {
        Self { lanes: Vec::new() }
    }

    pub fn get_lanes(&self) -> &[Option<Oid>] {
        &self.lanes
    }

    /// assign commit to a lane and update lanes for parents
    pub fn assign_commit(&mut self, commit_oid: &Oid, parent_oids: &[Oid]) -> usize {
        // 1️⃣ หา lane ของ commit
        let mut lane = match self
            .lanes
            .iter()
            .position(|slot| slot.as_ref() == Some(commit_oid))
        {
            Some(i) => i,
            None => {
                self.lanes.push(None);
                self.lanes.len() - 1
            }
        };

        // 2️⃣ consume commit
        self.lanes[lane] = None;

        // 3️⃣ หา parent ที่มี lane อยู่แล้ว
        let mut continue_parent = None;
        let mut parent_lane = None;

        for parent in parent_oids {
            if let Some(idx) = self
                .lanes
                .iter()
                .position(|slot| slot.as_ref() == Some(parent))
            {
                continue_parent = Some(*parent);
                parent_lane = Some(idx);
                break;
            }
        }

        // 4️⃣ ถ้ามี parent ที่อยู่ใน lane อื่น → merge
        if let (Some(parent), Some(p_lane)) = (continue_parent, parent_lane) {
            // ลบ lane ของ parent ก่อน
            if p_lane < lane {
                self.lanes.remove(p_lane);
                lane -= 1; // 🔑 ปรับ lane!
            } else if p_lane > lane {
                self.lanes.remove(p_lane);
            }

            self.lanes[lane] = Some(parent);
        }
        // 5️⃣ ไม่มี parent ใน lane → ใช้ parent ตัวแรก
        else if let Some(parent) = parent_oids.first() {
            self.lanes[lane] = Some(*parent);
        }
        // else → ไม่มี parent → lane ปิด

        // 6️⃣ parent ที่เหลือ เปิด lane ใหม่ (กันซ้ำ)
        for parent in parent_oids {
            if Some(*parent) != self.lanes[lane]
                && !self.lanes.iter().any(|s| s.as_ref() == Some(parent))
            {
                self.lanes.push(Some(*parent));
            }
        }

        // 7️⃣ cleanup lane ว่างท้าย
        while matches!(self.lanes.last(), Some(None)) {
            self.lanes.pop();
        }

        lane
    }
}
