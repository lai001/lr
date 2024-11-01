#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotator {
    pub yaw: f32,
    pub roll: f32,
    pub pitch: f32,
}

impl Rotator {
    pub fn zero() -> Rotator {
        Rotator {
            yaw: 0.0,
            roll: 0.0,
            pitch: 0.0,
        }
    }

    pub fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_euler(glam::EulerRot::XYZ, self.pitch, self.yaw, self.roll)
    }

    pub fn from_matrix(matrix: &glam::Mat4) -> Self {
        let (scale, rotation, _) = matrix.to_scale_rotation_translation();
        let matrix = glam::Mat4::from_scale_rotation_translation(scale, rotation, glam::Vec3::ZERO);
        let (_, quat_rotation, _) = matrix.to_scale_rotation_translation();
        let (pitch, yaw, roll) = quat_rotation.to_euler(glam::EulerRot::XYZ);
        Rotator { yaw, roll, pitch }
    }

    pub fn to_radians(&self) -> Self {
        Rotator {
            yaw: self.yaw.to_radians(),
            roll: self.roll.to_radians(),
            pitch: self.pitch.to_radians(),
        }
    }

    pub fn to_degrees(&self) -> Self {
        Rotator {
            yaw: self.yaw.to_degrees(),
            roll: self.roll.to_degrees(),
            pitch: self.pitch.to_degrees(),
        }
    }

    pub fn to_forward_vector(&self) -> glam::Vec3 {
        let mut forward_vector = glam::Vec3::ZERO;
        let pitch = -self.pitch;
        forward_vector.x = pitch.cos() * self.yaw.sin();
        forward_vector.y = pitch.sin();
        forward_vector.z = pitch.cos() * self.yaw.cos();
        forward_vector
        // glam::Mat4::from_euler(glam::EulerRot::XYZ, self.pitch, self.yaw, self.roll)
        //     .transform_vector3(crate::misc::FORWARD_VECTOR)
    }

    pub fn from_forward_vector(forward_vector: glam::Vec3) -> Rotator {
        let pitch = (-forward_vector.y).asin();
        let yaw = forward_vector.x.atan2(forward_vector.z);
        Rotator {
            yaw,
            roll: 0.0,
            pitch,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Rotator;
    use crate::misc::FORWARD_VECTOR;
    use glam::{BVec3, BVec4A, EulerRot};

    #[test]
    fn test_rotator() {
        let transformation =
            glam::Mat4::from_euler(EulerRot::XYZ, 1.0, -1.0, 1.0) * glam::Mat4::IDENTITY;
        let rotator = Rotator::from_matrix(&transformation);
        assert_eq!(
            (rotator.to_matrix().x_axis - transformation.x_axis)
                .abs()
                .cmple(glam::Vec4::splat(0.001)),
            BVec4A::splat(true)
        );
        assert_eq!(
            (rotator.to_matrix().y_axis - transformation.y_axis)
                .abs()
                .cmple(glam::Vec4::splat(0.001)),
            BVec4A::splat(true)
        );
        assert_eq!(
            (rotator.to_matrix().z_axis - transformation.z_axis)
                .abs()
                .cmple(glam::Vec4::splat(0.001)),
            BVec4A::splat(true)
        );
        assert_eq!(
            (rotator.to_matrix().w_axis - transformation.w_axis)
                .abs()
                .cmple(glam::Vec4::splat(0.001)),
            BVec4A::splat(true)
        );
    }

    #[test]
    fn test_rotator_1() {
        let transformation = glam::Mat4::IDENTITY;
        let rotator = Rotator::from_matrix(&transformation);
        let forward_vector = rotator.to_forward_vector();
        assert_eq!(forward_vector, FORWARD_VECTOR);
    }

    #[test]
    fn test_rotator_2() {
        let transformation =
            glam::Mat4::from_rotation_x(45.0_f32.to_radians()) * glam::Mat4::IDENTITY;
        let rotator = Rotator::from_matrix(&transformation);
        let forward_vector = rotator.to_forward_vector();
        assert_eq!(
            (forward_vector - glam::vec3(0.0, -0.7071068, 0.7071067))
                .abs()
                .cmple(glam::Vec3::splat(0.001)),
            BVec3::splat(true)
        );
    }

    #[test]
    fn test_rotator_3() {
        let transformation =
            glam::Mat4::from_rotation_x(45.0_f32.to_radians()) * glam::Mat4::IDENTITY;
        let rotator = Rotator::from_matrix(&transformation);
        let forward_vector = rotator.to_forward_vector();
        assert_eq!(rotator, Rotator::from_forward_vector(forward_vector));
    }

    #[test]
    fn test_rotator_4() {
        let transformation =
            glam::Mat4::from_rotation_y(45.0_f32.to_radians()) * glam::Mat4::IDENTITY;
        let rotator = Rotator::from_matrix(&transformation);
        let forward_vector = rotator.to_forward_vector();
        assert_eq!(rotator, Rotator::from_forward_vector(forward_vector));
    }
}
