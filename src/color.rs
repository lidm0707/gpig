use std::collections::HashMap;

pub struct ColorManager {
    count_color: usize,
    map_color: HashMap<usize, usize>,
    colors: Vec<u32>,
}

impl ColorManager {
    pub fn new(colors: Vec<u32>) -> Self {
        ColorManager {
            count_color: 0,
            map_color: HashMap::new(),
            colors,
        }
    }

    pub fn get_color(&mut self, lane: &usize) -> usize {
        self.count_color += 1;
        let color = match self.map_color.get(lane) {
            Some(color) => *color,
            _ => {
                if self.count_color < self.colors.len() {
                    self.count_color
                } else {
                    self.count_color = 0;
                    self.count_color
                }
            }
        };
        self.map_color.insert(*lane, color);
        color
    }

    pub fn remove_lane_color(&mut self, lane: &usize) {
        self.map_color.remove(lane);
    }
}
