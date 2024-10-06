pub const FORWARD_VECTOR: glam::Vec3 = glam::Vec3::Z;
pub const UP_VECTOR: glam::Vec3 = glam::Vec3::Y;

fn compute_forward_vector(transformation: &glam::Mat4) -> glam::Vec3 {
    crate::rotator::Rotator::from_matrix(transformation).to_forward_vector()
}

pub trait Mat4Extension {
    fn get_forward_vector(&self) -> glam::Vec3;
}

impl Mat4Extension for glam::Mat4 {
    fn get_forward_vector(&self) -> glam::Vec3 {
        compute_forward_vector(self)
    }
}
