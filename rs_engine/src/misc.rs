use rapier3d::na::Point3;
use rs_core_minimal::sphere_3d::Sphere3D;
pub const FORWARD_VECTOR: glam::Vec3 = glam::Vec3::Z;
pub const UP_VECTOR: glam::Vec3 = glam::Vec3::Y;
pub const RIGHT_VECTOR: glam::Vec3 = glam::Vec3::X;

fn compute_forward_vector(transformation: &glam::Mat4) -> glam::Vec3 {
    transformation.transform_vector3(FORWARD_VECTOR)
    // crate::rotator::Rotator::from_matrix(transformation).to_forward_vector()
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

pub fn points_to_aabb(points: &[glam::Vec3]) -> rapier3d::prelude::Aabb {
    let points: Vec<rapier3d::math::Point<f32>> = points
        .iter()
        .map(|x| rapier3d::math::Point::<f32>::from_slice(&x.to_array()))
        .collect();

    let aabb = rapier3d::prelude::Aabb::from_points(&points);
    aabb
}

pub fn static_mesh_get_aabb(
    static_mesh: &rs_artifact::static_mesh::StaticMesh,
) -> rapier3d::prelude::Aabb {
    let points: Vec<glam::Vec3> = static_mesh
        .vertexes
        .iter()
        .map(|x| x.position.clone())
        .collect();
    points_to_aabb(&points)
}

pub fn transform_aabb(
    aabb: &rapier3d::prelude::Aabb,
    transformation: &glam::Mat4,
) -> rapier3d::prelude::Aabb {
    let mins = glam::vec3(aabb.mins.x, aabb.mins.y, aabb.mins.z);
    let maxs = glam::vec3(aabb.maxs.x, aabb.maxs.y, aabb.maxs.z);
    let mins = transformation.transform_point3(mins);
    let maxs = transformation.transform_point3(maxs);
    rapier3d::prelude::Aabb::new(
        Point3::from_slice(&mins.to_array()),
        Point3::from_slice(&maxs.to_array()),
    )
}

pub fn merge_aabb(aabbs: &[rapier3d::prelude::Aabb]) -> Option<rapier3d::prelude::Aabb> {
    if aabbs.is_empty() {
        return None;
    }
    let mut points: Vec<glam::Vec3> = Vec::with_capacity(2 * aabbs.len());
    for aabb in aabbs.iter() {
        let mins = glam::vec3(aabb.mins.x, aabb.mins.y, aabb.mins.z);
        let maxs = glam::vec3(aabb.maxs.x, aabb.maxs.y, aabb.maxs.z);
        points.push(mins);
        points.push(maxs);
    }
    Some(points_to_aabb(&points))
}

pub fn aabb_as_sphere(aabb: &rapier3d::prelude::Aabb) -> Sphere3D {
    let center = aabb.center();
    let center = glam::vec3(center.x, center.y, center.z);
    let half_extents = glam::Vec3::from_slice(aabb.half_extents().as_slice());
    Sphere3D::new(center, half_extents.length())
}

pub fn compute_appropriate_offset_look_and_projection_matrix(
    level: &crate::content::level::Level,
) -> Option<(f32, glam::Vec3, glam::Mat4)> {
    if let Some(aabb) = level.compute_scene_aabb() {
        let sphere = aabb_as_sphere(&aabb);
        let size = sphere.radius;
        let directional_light_projection =
            glam::Mat4::orthographic_rh(-size, size, -size, size, 0.01, 0.01 + size * 2.0);
        Some((size, sphere.center, directional_light_projection))
    } else {
        None
    }
}

pub trait Mat4Extension {
    fn get_forward_vector(&self) -> glam::Vec3;
    fn remove_translation(&self) -> glam::Mat4;
    fn slerp(&self, rhs: &Self, s: f32) -> glam::Mat4;
    fn lerp(&self, rhs: &Self, s: f32) -> glam::Mat4;
}

impl Mat4Extension for glam::Mat4 {
    fn get_forward_vector(&self) -> glam::Vec3 {
        compute_forward_vector(self)
    }

    fn remove_translation(&self) -> glam::Mat4 {
        let (scale, rotation, _) = self.to_scale_rotation_translation();
        glam::Mat4::from_scale_rotation_translation(scale, rotation, glam::Vec3::ZERO)
    }

    fn slerp(&self, rhs: &Self, s: f32) -> glam::Mat4 {
        let lhs = self.to_scale_rotation_translation();
        let rhs = rhs.to_scale_rotation_translation();
        glam::Mat4::from_scale_rotation_translation(
            lhs.0.lerp(rhs.0, s),
            lhs.1.slerp(rhs.1, s),
            lhs.2.lerp(rhs.2, s),
        )
    }

    fn lerp(&self, rhs: &Self, s: f32) -> glam::Mat4 {
        let lhs = self.to_scale_rotation_translation();
        let rhs = rhs.to_scale_rotation_translation();
        glam::Mat4::from_scale_rotation_translation(
            lhs.0.lerp(rhs.0, s),
            lhs.1.lerp(rhs.1, s),
            lhs.2.lerp(rhs.2, s),
        )
    }
}
