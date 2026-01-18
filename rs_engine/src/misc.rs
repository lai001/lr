use rs_core_minimal::{
    misc::distance_from_point_to_segment, parallel::ComputeDispatcher, sphere_3d::Sphere3D,
};
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

pub fn points_to_aabb(points: Vec<glam::Vec3>) -> rapier3d::prelude::Aabb {
    let aabb = rapier3d::prelude::Aabb::from_points(points);
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
    points_to_aabb(points)
}

pub fn transform_aabb(
    aabb: &rapier3d::prelude::Aabb,
    transformation: &glam::Mat4,
) -> rapier3d::prelude::Aabb {
    let mins = glam::vec3(aabb.mins.x, aabb.mins.y, aabb.mins.z);
    let maxs = glam::vec3(aabb.maxs.x, aabb.maxs.y, aabb.maxs.z);
    let mins = transformation.transform_point3(mins);
    let maxs = transformation.transform_point3(maxs);
    rapier3d::prelude::Aabb::new(mins, maxs)
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
    Some(points_to_aabb(points))
}

pub fn aabb_as_sphere(aabb: &rapier3d::prelude::Aabb) -> Sphere3D {
    let center = aabb.center();
    let center = glam::vec3(center.x, center.y, center.z);
    let half_extents = aabb.half_extents();
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

pub fn sdf_from_polygon(
    polygon: Vec<glam::Vec2>,
    image_size: glam::UVec2,
    is_reverse: bool,
) -> image::Rgb32FImage {
    struct UnsafeWrapperType(*mut image::Rgb32FImage);
    unsafe impl Send for UnsafeWrapperType {}
    unsafe impl Sync for UnsafeWrapperType {}

    let size = image_size.extend(1);
    let mut image = image::Rgb32FImage::new(size.x, size.y);
    let raw_image = (&mut image) as *mut image::Rgb32FImage;
    let wrapper_type = std::sync::Arc::new(UnsafeWrapperType(raw_image));
    let workgroup_size = glam::UVec2::splat(32).extend(1);
    let num_work_groups = ComputeDispatcher::estimate_num_work_groups(&size, &workgroup_size);
    rs_core_minimal::parallel::ComputeDispatcher::new(workgroup_size).dispatch_workgroups(
        num_work_groups,
        {
            move |_, _, dispatch_thread_id, _| {
                let image = unsafe { wrapper_type.0.as_mut().unwrap() };
                if let Some(pixel) =
                    image.get_pixel_mut_checked(dispatch_thread_id.x, dispatch_thread_id.y)
                {
                    let p = glam::vec2(dispatch_thread_id.x as f32, dispatch_thread_id.y as f32);
                    let is_inside = rs_core_minimal::misc::is_point_in_polygon(p, &polygon, true);
                    let mut min_distance = std::f32::MAX;
                    for i in 0..polygon.len() {
                        let current = polygon[i];
                        let next = polygon[(i + 1) % polygon.len()];
                        let distance = distance_from_point_to_segment(current, next, p);
                        min_distance = min_distance.min(distance);
                    }

                    let mut color = glam::vec3(min_distance, min_distance, min_distance);
                    if is_reverse {
                        if !is_inside {
                            color *= -1.0;
                        }
                    } else {
                        if is_inside {
                            color *= -1.0;
                        }
                    }

                    *pixel = image::Rgb::<f32>(color.to_array());
                }
            }
        },
    );

    image
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

#[cfg(test)]
mod test {
    use super::sdf_from_polygon;

    #[test]
    fn sdf_from_polygon_test() {
        let polygon =
            rs_core_minimal::misc::generate_circle_points(glam::vec2(2048.0, 2048.0), 1024.0, 256);
        let image = sdf_from_polygon(polygon, glam::uvec2(4096, 4096), false);
        assert_eq!(
            *image.get_pixel(0, 2048),
            image::Rgb::<f32>(glam::Vec3::splat(1024.0).to_array())
        );
    }
}
