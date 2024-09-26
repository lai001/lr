pub struct Line {
    pub p_0: glam::Vec3,
    pub p_1: glam::Vec3,
}

#[derive(Debug)]
pub struct Frustum {
    pub near_0: glam::Vec3,
    pub near_1: glam::Vec3,
    pub near_2: glam::Vec3,
    pub near_3: glam::Vec3,
    pub far_0: glam::Vec3,
    pub far_1: glam::Vec3,
    pub far_2: glam::Vec3,
    pub far_3: glam::Vec3,
}

impl Frustum {
    pub fn make_lines(&self) -> Vec<Line> {
        let mut near_lines = Self::lines_from_points(vec![
            self.near_0,
            self.near_1,
            self.near_2,
            self.near_3,
            self.near_0,
        ]);
        let mut far_lines = Self::lines_from_points(vec![
            self.far_0, self.far_1, self.far_2, self.far_3, self.far_0,
        ]);
        let mut tr_lines = Self::lines_from_points(vec![self.near_0, self.far_0]);
        let mut br_lines = Self::lines_from_points(vec![self.near_1, self.far_1]);
        let mut tl_lines = Self::lines_from_points(vec![self.near_2, self.far_2]);
        let mut bl_lines = Self::lines_from_points(vec![self.near_3, self.far_3]);
        let mut lines = vec![];

        lines.append(&mut near_lines);
        lines.append(&mut far_lines);
        lines.append(&mut tr_lines);
        lines.append(&mut br_lines);
        lines.append(&mut tl_lines);
        lines.append(&mut bl_lines);

        lines
    }

    fn lines_from_points(points: Vec<glam::Vec3>) -> Vec<Line> {
        points
            .windows(2)
            .map(|x| Line {
                p_0: x[0],
                p_1: x[1],
            })
            .collect()
    }
}
