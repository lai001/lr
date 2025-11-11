use crate::rect::Rect;

#[derive(Debug, Clone)]
pub struct SkylineNode {
    pub x: u32,
    pub y: u32,
    pub width: u32,
}

pub struct SkylineBinPack {
    pub bin_width: u32,
    pub bin_height: u32,
    pub skyline: Vec<SkylineNode>,
}

impl SkylineBinPack {
    pub fn new(bin_width: u32, bin_height: u32) -> Self {
        Self {
            bin_width,
            bin_height,
            skyline: vec![SkylineNode {
                x: 0,
                y: 0,
                width: bin_width,
            }],
        }
    }

    pub fn insert(&mut self, width: u32, height: u32) -> Option<Rect> {
        let mut best_y = u32::MAX;
        let mut best_x = 0;
        let mut best_index = None;

        for (i, node) in self.skyline.iter().enumerate() {
            if let Some(y) = self.fit(i, width, height) {
                if y < best_y || (y == best_y && node.x < best_x) {
                    best_y = y;
                    best_x = node.x;
                    best_index = Some(i);
                }
            }
        }

        if let Some(index) = best_index {
            self.add_skyline_level(index, best_x, best_y, width, height);
            Some(Rect {
                x: best_x,
                y: best_y,
                width,
                height,
            })
        } else {
            None
        }
    }

    fn fit(&self, index: usize, width: u32, height: u32) -> Option<u32> {
        let x = self.skyline[index].x;
        let mut y = self.skyline[index].y;
        let mut width_left = width;

        let mut i = index;
        if x + width > self.bin_width {
            return None;
        }

        while width_left > 0 {
            if i >= self.skyline.len() {
                return None;
            }
            if self.skyline[i].y > y {
                y = self.skyline[i].y;
            }
            if y + height > self.bin_height {
                return None;
            }
            width_left = width_left.saturating_sub(self.skyline[i].width);
            i += 1;
        }

        Some(y)
    }

    fn add_skyline_level(&mut self, index: usize, x: u32, y: u32, width: u32, height: u32) {
        let new_node = SkylineNode {
            x,
            y: y + height,
            width,
        };
        self.skyline.insert(index, new_node);

        let i = index + 1;
        while i < self.skyline.len() {
            let node = &self.skyline[i];
            if node.x < x + width {
                let shrink = (x + width).saturating_sub(node.x);
                if shrink >= node.width {
                    self.skyline.remove(i);
                } else {
                    self.skyline[i].x += shrink;
                    self.skyline[i].width -= shrink;
                    break;
                }
            } else {
                break;
            }
        }

        self.merge();
    }

    fn merge(&mut self) {
        let mut i = 0;
        while i + 1 < self.skyline.len() {
            if self.skyline[i].y == self.skyline[i + 1].y {
                self.skyline[i].width += self.skyline[i + 1].width;
                self.skyline.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }
}
