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

    pub fn signed_distance_to_point(&self, target_point: glam::Vec3) -> f32 {
        // target_point.dot(self.normal_vector) - self.point.distance(glam::Vec3::ZERO)
        let x = glam::vec4(
            self.normal_vector.x,
            self.normal_vector.y,
            self.normal_vector.z,
            -self.normal_vector.dot(self.point),
        );
        let y = glam::vec4(target_point.x, target_point.y, target_point.z, 1.0);
        x.dot(y)
    }

    pub fn is_normal_side(&self, target_point: glam::Vec3) -> bool {
        let direction = target_point - self.point;
        direction.dot(self.normal_vector) > 0.0
    }

    pub fn is_inside(&self, sphere3d: &crate::sphere_3d::Sphere3D) -> bool {
        let signed_distance = self.signed_distance_to_point(sphere3d.center);
        if signed_distance > 0.0 && signed_distance > sphere3d.radius {}
        if signed_distance > sphere3d.radius {
            return false;
        } else {
            return true;
        }
    }
}

#[cfg(test)]
mod test {
    use super::Plane3D;

    #[test]
    fn distance_to_point_test() {
        let plane = Plane3D::new(glam::Vec3::Y, glam::Vec3::ZERO);
        let distance = plane.signed_distance_to_point(glam::vec3(1.0, 5.0, 100.0));
        assert_eq!(5.0, distance);
        let distance = plane.signed_distance_to_point(glam::vec3(0.0, 5.0, 0.0));
        assert_eq!(5.0, distance);
        let distance = plane.signed_distance_to_point(glam::vec3(0.0, -5.0, 0.0));
        assert_eq!(-5.0, distance);
    }

    #[test]
    fn is_normal_side_test() {
        let plane = Plane3D::new(glam::Vec3::Y, glam::Vec3::ZERO);
        let is_normal_side = plane.is_normal_side(glam::vec3(1.0, 5.0, 100.0));
        assert_eq!(true, is_normal_side);
        let is_normal_side = plane.is_normal_side(glam::vec3(1.0, -5.0, 100.0));
        assert_eq!(false, is_normal_side);
    }
}
