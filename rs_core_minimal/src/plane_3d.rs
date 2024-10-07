use crate::line_3d::Line3D;

pub struct Plane3D {
    // a: f32,
    // b: f32,
    // c: f32,
    // d: f32,
    pub normal_vector: glam::Vec3,
    pub point: glam::Vec3,
}

impl Plane3D {
    pub fn new(normal_vector: glam::Vec3, point: glam::Vec3) -> Plane3D {
        // let normal = normal_vector.normalize();
        // let a = normal_vector.x;
        // let b = normal_vector.y;
        // let c = normal_vector.z;
        // let d = -normal_vector.dot(point);
        // Plane3D { a, b, c, d }
        Plane3D {
            normal_vector,
            point,
        }
    }

    pub fn intersection_line_two_points(
        &self,
        start: glam::Vec3,
        end: glam::Vec3,
    ) -> Option<glam::Vec3> {
        let line = Line3D::from_points(start, end);
        let denominator = line.direction.dot(self.normal_vector);
        let numerator = (self.point - line.point).dot(self.normal_vector);
        if denominator == 0.0 {
            if numerator == 0.0 {
                return None;
            } else {
                return None;
            }
        } else {
            let d = numerator / denominator;
            return Some(line.direction * d + line.point);
        }
    }
}
