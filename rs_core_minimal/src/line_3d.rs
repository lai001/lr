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

pub struct LineSegment3D {
    pub start: glam::Vec3,
    pub end: glam::Vec3,
}

impl LineSegment3D {
    pub fn find_ratio(&self, p: &glam::Vec3) -> f32 {
        let u = p - self.start;
        let v = self.end - self.start;
        let cast = (u.dot(v) / v.length().powf(2.0)) * v;
        cast.normalize().dot(v.normalize()) * cast.length() / v.length()
    }
}

#[cfg(test)]
mod test {
    use super::LineSegment3D;

    #[test]
    fn find_ratio_test() {
        let line = LineSegment3D {
            start: glam::vec3(0.0, 0.0, 0.0),
            end: glam::vec3(5.0, 0.0, 0.0),
        };
        let ratio = line.find_ratio(&glam::vec3(1.0, 1.0, 0.0));
        assert_eq!(ratio, 0.2);
        let ratio = line.find_ratio(&glam::vec3(-1.0, 1.0, 0.0));
        assert_eq!(ratio, -0.2);
    }
}
