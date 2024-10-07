pub struct Line3D {
    pub direction: glam::Vec3,
    pub point: glam::Vec3,
}

impl Line3D {
    pub fn from_points(p1: glam::Vec3, p2: glam::Vec3) -> Line3D {
        let direction = p2 - p1;
        Line3D {
            direction,
            point: p1,
        }
    }
}
