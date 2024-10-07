pub const FORWARD_VECTOR: glam::Vec3 = glam::Vec3::Z;
pub const UP_VECTOR: glam::Vec3 = glam::Vec3::Y;
pub const RIGHT_VECTOR: glam::Vec3 = glam::Vec3::X;

fn compute_forward_vector(transformation: &glam::Mat4) -> glam::Vec3 {
    crate::rotator::Rotator::from_matrix(transformation).to_forward_vector()
}

pub fn project_to_world(
    cursor_position: &glam::Vec2,
    window_size: &glam::Vec2,
    camera_view_matrix: glam::Mat4,
    camera_projection_matrix: glam::Mat4,
) -> (glam::Vec3, glam::Vec3) {
    let ndc_cursor = glam::vec2(
        cursor_position.x / window_size.x * 2.0 - 1.0,
        1.0 - cursor_position.y / window_size.y * 2.0,
    );
    let ndc_to_world = camera_projection_matrix * camera_view_matrix;
    let ndc_to_world = ndc_to_world.inverse();
    let start = ndc_to_world.project_point3(glam::vec3(ndc_cursor.x, ndc_cursor.y, 0.0));
    let end = ndc_to_world.project_point3(glam::vec3(ndc_cursor.x, ndc_cursor.y, 1.0));
    (start, end)
}

pub trait Mat4Extension {
    fn get_forward_vector(&self) -> glam::Vec3;
    fn remove_translation(&self) -> glam::Mat4;
}

impl Mat4Extension for glam::Mat4 {
    fn get_forward_vector(&self) -> glam::Vec3 {
        compute_forward_vector(self)
    }

    fn remove_translation(&self) -> glam::Mat4 {
        let (scale, rotation, _) = self.to_scale_rotation_translation();
        glam::Mat4::from_scale_rotation_translation(scale, rotation, glam::Vec3::ZERO)
    }
}
