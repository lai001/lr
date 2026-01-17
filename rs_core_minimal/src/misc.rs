use crate::frustum::{Frustum, FrustumPlanes};
use std::sync::Arc;

pub fn calculate_max_mips(length: u32) -> u32 {
    32 - length.leading_zeros()
    // let mut mipmap_level: u32 = 1;
    // let mut length = length;
    // while length > 4 {
    //     length /= 2;
    //     mipmap_level += 1;
    // }
    // return mipmap_level;
}

pub fn calculate_mipmap_level_sizes(length: u32) -> Vec<u32> {
    let mut sizes = Vec::new();
    let mut length = length;
    while length > 0 {
        sizes.push(length);
        length /= 2;
    }
    sizes
}

pub fn get_mip_level_size(length: u32, level: u32) -> u32 {
    u32::max(1, length >> level)
}

#[cfg(feature = "editor")]
pub fn is_run_from_ide() -> bool {
    let vars = std::env::vars().filter(|x| x.0 == "VSCODE_HANDLES_UNCAUGHT_ERRORS".to_string());
    vars.count() != 0
}

#[cfg(feature = "editor")]
pub fn is_dev_mode() -> bool {
    let vars = std::env::vars().filter(|x| x.0 == "VSCODE_PID".to_string());
    vars.count() != 0
}

pub fn get_md5_from_string(text: &str) -> String {
    let mut hasher = <md5::Md5 as md5::Digest>::new();
    md5::digest::Update::update(&mut hasher, text.as_bytes());
    let result = md5::Digest::finalize(hasher);
    let result = result.to_ascii_lowercase();
    let result = result
        .iter()
        .fold("".to_string(), |acc, x| format!("{acc}{:x?}", x));
    result
}

pub fn get_md5_from_reader<R: std::io::Read>(reader: &mut R) -> String {
    let mut buf: Vec<u8> = vec![];
    let _ = reader.read_to_end(&mut buf);
    let mut hasher = <md5::Md5 as md5::Digest>::new();
    md5::Digest::update(&mut hasher, buf);
    let result = md5::Digest::finalize(hasher);
    let result = result.to_ascii_lowercase();
    let result = result
        .iter()
        .fold("".to_string(), |acc, x| format!("{acc}{:x?}", x));
    result
}

pub fn get_md5_from_buf(buf: &Vec<u8>) -> String {
    let mut cursor = std::io::Cursor::new(buf);
    get_md5_from_reader(&mut cursor)
}

pub fn get_sha256_from_reader<R: std::io::Read>(reader: &mut R) -> String {
    let mut buf: Vec<u8> = vec![];
    let _ = reader.read_to_end(&mut buf);
    let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
    sha2::Digest::update(&mut hasher, buf);
    let result = sha2::Digest::finalize(hasher);
    let result = result.to_ascii_lowercase();
    let result = result
        .iter()
        .fold("".to_string(), |acc, x| format!("{acc}{:x?}", x));
    result
}

pub fn get_sha256_from_buf(buf: &Vec<u8>) -> String {
    let mut cursor = std::io::Cursor::new(buf);
    get_sha256_from_reader(&mut cursor)
}

// fn transform_coordinates(p: glam::Vec3, m: glam::Mat4) -> glam::Vec3 {
//     let p = glam::vec4(p.x, p.y, p.z, 1.0);
//     (m * p).xyz()
// }

pub fn get_orthographic_frustum(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Frustum {
    // let projection = glam::Mat4::orthographic_rh(left, right, bottom, top, near, far);
    // let inv_projection = projection.inverse();

    let min = glam::vec3(left, bottom, near);
    let max = glam::vec3(right, top, far);
    let n_0 = glam::vec3(max.x, max.y, min.z);
    let n_1 = glam::vec3(max.x, min.y, min.z);
    let n_2 = glam::vec3(min.x, min.y, min.z);
    let n_3 = glam::vec3(min.x, max.y, min.z);

    let near_0 = n_0; //inv_projection.transform_point3(n_0);
    let near_1 = n_1; //inv_projection.transform_point3(n_1);
    let near_2 = n_2; //inv_projection.transform_point3(n_2);
    let near_3 = n_3; //inv_projection.transform_point3(n_3);

    let f_0 = glam::vec3(max.x, max.y, max.z);
    let f_1 = glam::vec3(max.x, min.y, max.z);
    let f_2 = glam::vec3(min.x, min.y, max.z);
    let f_3 = glam::vec3(min.x, max.y, max.z);

    let far_0 = f_0; //inv_projection.transform_point3(f_0);
    let far_1 = f_1; //inv_projection.transform_point3(f_1);
    let far_2 = f_2; //inv_projection.transform_point3(f_2);
    let far_3 = f_3; //inv_projection.transform_point3(f_3);

    Frustum {
        near_0,
        near_1,
        near_2,
        near_3,
        far_0,
        far_1,
        far_2,
        far_3,
    }
}

pub fn frustum_from_perspective(
    fov_y_radians: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
) -> Frustum {
    let near_top = z_near * (fov_y_radians / 2.0).tan();
    let near_bottom = -near_top;
    let near_right = near_top * aspect_ratio;
    let near_left = -near_right;

    let far_top = z_far * (fov_y_radians / 2.0).tan();
    let far_bottom = -far_top;
    let far_right = far_top * aspect_ratio;
    let far_left = -far_right;

    let near_0 = glam::vec3(near_right, near_top, z_near);
    let near_1 = glam::vec3(near_right, near_bottom, z_near);
    let near_2 = glam::vec3(near_left, near_bottom, z_near);
    let near_3 = glam::vec3(near_left, near_top, z_near);

    let far_0 = glam::vec3(far_right, far_top, z_far);
    let far_1 = glam::vec3(far_right, far_bottom, z_far);
    let far_2 = glam::vec3(far_left, far_bottom, z_far);
    let far_3 = glam::vec3(far_left, far_top, z_far);

    Frustum {
        near_0,
        near_1,
        near_2,
        near_3,
        far_0,
        far_1,
        far_2,
        far_3,
    }
}

pub fn is_valid_name(name: &str) -> bool {
    let re = regex::Regex::new(r"^\w+$").unwrap();
    re.is_match(name)
}

pub fn subdivide_two_points(subdivide: usize, p0: &glam::Vec3, p1: &glam::Vec3) -> Vec<glam::Vec3> {
    let mut points = Vec::with_capacity(2 + subdivide);
    points.push(*p0);
    for i in 0..subdivide {
        let point = p0.lerp(*p1, (1.0 / (subdivide + 1) as f32) * (i + 1) as f32);
        points.push(point);
    }
    points.push(*p1);
    points
}

pub fn subdivide_four_points(
    subdivide_i: usize,
    subdivide_j: usize,
    p0: &glam::Vec3,
    p1: &glam::Vec3,
    p2: &glam::Vec3,
    p3: &glam::Vec3,
) -> Vec<(glam::Vec3, glam::Vec3, glam::Vec3, glam::Vec3)> {
    let mut plane_points = Vec::with_capacity((subdivide_i + 1) * (subdivide_j + 1));
    let points0 = subdivide_two_points(subdivide_j, p0, p1);
    let points2 = subdivide_two_points(subdivide_j, p3, p2);
    for (lhs, rhs) in points2.windows(2).zip(points0.windows(2)) {
        let first_line = subdivide_two_points(subdivide_i, &lhs[0], &rhs[0]);
        let second_line = subdivide_two_points(subdivide_i, &lhs[1], &rhs[1]);
        for (first_line, second_line) in first_line.windows(2).zip(second_line.windows(2)) {
            let plane = (first_line[1], second_line[1], second_line[0], first_line[0]);
            plane_points.push(plane);
        }
    }
    plane_points
}

fn split_frustum_multiple_thread(
    frustum: &Frustum,
    subdivide_i: usize,
    subdivide_j: usize,
    subdivide_k: usize,
) -> Vec<Frustum> {
    let points0 = Arc::new(subdivide_two_points(
        subdivide_k,
        &frustum.near_0,
        &frustum.far_0,
    ));
    let points2 = Arc::new(subdivide_two_points(
        subdivide_k,
        &frustum.near_2,
        &frustum.far_2,
    ));
    let points3 = Arc::new(subdivide_two_points(
        subdivide_k,
        &frustum.near_3,
        &frustum.far_3,
    ));

    #[derive(Clone)]
    struct TaskResult {
        index: usize,
        frustum: Frustum,
    }

    let (sender, receiver) = std::sync::mpsc::channel::<TaskResult>();
    for k in 0..subdivide_k + 1 {
        for j in 0..subdivide_j + 1 {
            for i in 0..subdivide_i + 1 {
                crate::thread_pool::ThreadPool::global().spawn({
                    let index =
                        k * ((subdivide_i + 1) * (subdivide_j + 1)) + j * (subdivide_i + 1) + i;
                    let sender = sender.clone();
                    let points0 = points0.clone();
                    let points2 = points2.clone();
                    let points3 = points3.clone();
                    move || {
                        let step_horizontal_near =
                            (points0[k] - points3[k]) / (subdivide_i + 1) as f32;
                        let step_vertical_near =
                            (points2[k] - points3[k]) / (subdivide_j + 1) as f32;

                        let step_horizontal_far =
                            (points0[k + 1] - points3[k + 1]) / (subdivide_i + 1) as f32;
                        let step_vertical_far =
                            (points2[k + 1] - points3[k + 1]) / (subdivide_j + 1) as f32;

                        let near_3 = points3[k]
                            + step_horizontal_near * (i as f32)
                            + step_vertical_near * (j as f32);
                        let near_0 = near_3 + step_horizontal_near;
                        let near_2 = near_3 + step_vertical_near;
                        let near_1 = near_0 + step_vertical_near;

                        let far_3 = points3[k + 1]
                            + step_horizontal_far * (i as f32)
                            + step_vertical_far * (j as f32);
                        let far_0 = far_3 + step_horizontal_far;
                        let far_2 = far_3 + step_vertical_far;
                        let far_1 = far_0 + step_vertical_far;

                        let _ = sender.send(TaskResult {
                            index,
                            frustum: Frustum {
                                near_0,
                                near_1,
                                near_2,
                                near_3,
                                far_0,
                                far_1,
                                far_2,
                                far_3,
                            },
                        });
                    }
                });
            }
        }
    }

    let mut results: Vec<Frustum> =
        vec![Frustum::default(); (subdivide_i + 1) * (subdivide_j + 1) * (subdivide_k + 1)];

    let mut done_task: usize = 0;

    while let Ok(task_result) = receiver.recv() {
        let index = task_result.index;
        results[index] = task_result.frustum;
        done_task += 1;
        if done_task == (subdivide_i + 1) * (subdivide_j + 1) * (subdivide_k + 1) {
            break;
        }
    }
    results
}

pub fn split_frustum(
    frustum: &Frustum,
    subdivide_i: usize,
    subdivide_j: usize,
    subdivide_k: usize,
) -> Vec<Frustum> {
    return split_frustum_multiple_thread(frustum, subdivide_i, subdivide_j, subdivide_k);
    // let mut clusters =
    //     Vec::with_capacity((subdivide_i + 1) * (subdivide_j + 1) * (subdivide_k + 1));

    // let points0 = subdivide_two_points(subdivide_k, &frustum.near_0, &frustum.far_0);
    // let points1 = subdivide_two_points(subdivide_k, &frustum.near_1, &frustum.far_1);
    // let points2 = subdivide_two_points(subdivide_k, &frustum.near_2, &frustum.far_2);
    // let points3 = subdivide_two_points(subdivide_k, &frustum.near_3, &frustum.far_3);

    // let zip = points0
    //     .windows(2)
    //     .zip(points1.windows(2))
    //     .zip(points2.windows(2))
    //     .zip(points3.windows(2));

    // for item in zip {
    //     let points3 = item.1;
    //     let points2 = item.0 .1;
    //     let points1 = item.0 .0 .1;
    //     let points0 = item.0 .0 .0;
    //     let near_planes = subdivide_four_points(
    //         subdivide_i,
    //         subdivide_j,
    //         &points0[0],
    //         &points1[0],
    //         &points2[0],
    //         &points3[0],
    //     );

    //     let far_planes = subdivide_four_points(
    //         subdivide_i,
    //         subdivide_j,
    //         &points0[1],
    //         &points1[1],
    //         &points2[1],
    //         &points3[1],
    //     );

    //     for (near_plane, far_plane) in near_planes.iter().zip(far_planes) {
    //         let frustum = Frustum {
    //             near_0: near_plane.0,
    //             near_1: near_plane.1,
    //             near_2: near_plane.2,
    //             near_3: near_plane.3,
    //             far_0: far_plane.0,
    //             far_1: far_plane.1,
    //             far_2: far_plane.2,
    //             far_3: far_plane.3,
    //         };
    //         clusters.push(frustum);
    //     }
    // }

    // clusters
}

pub fn point_light_radius(
    quadratic: f32,
    linear: f32,
    constant: f32,
    attenuation_threshold: f32,
) -> f32 {
    debug_assert_ne!(quadratic, 0.0);
    debug_assert!(attenuation_threshold > 0.0);
    let c = constant - (1.0 / attenuation_threshold);
    let delta = linear.powf(2.0) - 4.0 * quadratic * c;
    debug_assert!(delta >= 0.0);
    let x1 = (-linear + delta.sqrt()) / (2.0 * quadratic);
    let x2 = (-linear - delta.sqrt()) / (2.0 * quadratic);
    x1.max(x2)
}

pub fn is_sphere_visible_to_frustum(
    sphere3d: &crate::sphere_3d::Sphere3D,
    frustum: &Frustum,
) -> bool {
    let FrustumPlanes {
        left_plane,
        right_plane,
        top_plane,
        bottom_plane,
        front_plane,
        back_plane,
    } = FrustumPlanes::new(frustum);

    left_plane.is_inside(sphere3d)
        && right_plane.is_inside(sphere3d)
        && top_plane.is_inside(sphere3d)
        && bottom_plane.is_inside(sphere3d)
        && front_plane.is_inside(sphere3d)
        && back_plane.is_inside(sphere3d)
}

pub fn generate_circle_points(
    center: glam::Vec2,
    radius: f32,
    num_points: usize,
) -> Vec<glam::Vec2> {
    (0..num_points)
        .map(|i| {
            let theta = 2.0 * std::f32::consts::PI * (i as f32) / num_points as f32;
            glam::Vec2::new(
                center.x + radius * theta.cos(),
                center.y + radius * theta.sin(),
            )
        })
        .collect()
}

pub fn is_point_in_polygon(
    point: glam::Vec2,
    polygon: &[glam::Vec2],
    is_include_edge: bool,
) -> bool {
    let mut crossings = 0;
    for i in 0..polygon.len() {
        let current = polygon[i];
        let next = polygon[(i + 1) % polygon.len()];
        if is_include_edge {
            let is_on_edge = (point.y - current.y) * (next.x - current.x)
                == (next.y - current.y) * (point.x - current.x)
                && point.x >= current.x.min(next.x)
                && point.x <= current.x.max(next.x)
                && point.y >= current.y.min(next.y)
                && point.y <= current.y.max(next.y);
            if is_on_edge {
                return true;
            }
        }
        if (current.y > point.y) != (next.y > point.y) {
            let delta_x = next.x - current.x;
            let delta_y = next.y - current.y;
            let a = point.x - current.x;
            let c = point.y - current.y;
            let value = (a * delta_y - delta_x * c) * delta_y;
            if value < 0.0 {
                crossings += 1;
            }
        }
    }
    crossings % 2 == 1
}

pub fn distance_from_point_to_segment(a: glam::Vec2, b: glam::Vec2, p: glam::Vec2) -> f32 {
    if a == b {
        return p.distance(a);
    }
    let ab = b - a;
    let ap = p - a;
    let dot_ap_ab = ap.dot(ab);
    let dot_ab_ab = ab.dot(ab);
    if dot_ap_ab <= 0.0 {
        p.distance(a)
    } else if dot_ap_ab >= dot_ab_ab {
        p.distance(b)
    } else {
        let t = dot_ap_ab / dot_ab_ab;
        let projection = a + ab * t;
        p.distance(projection)
    }
}

pub fn get_git_hash() -> String {
    env!("GIT_HASH").to_string()
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::{
        distance_from_point_to_segment, frustum_from_perspective, generate_circle_points,
        is_sphere_visible_to_frustum, point_light_radius, split_frustum, subdivide_four_points,
        subdivide_two_points,
    };
    use crate::{
        misc::{get_sha256_from_reader, is_point_in_polygon, is_valid_name},
        sphere_3d::Sphere3D,
    };

    #[test]
    fn is_valid_name_test() {
        assert_eq!(is_valid_name("name"), true);
        assert_eq!(is_valid_name("_name"), true);
        assert_eq!(is_valid_name("name111"), true);
        assert_eq!(is_valid_name("name_111"), true);
        assert_eq!(is_valid_name("name_=111"), false);
        assert_eq!(is_valid_name("%name"), false);
        assert_eq!(is_valid_name(""), false);
        assert_eq!(is_valid_name("🔥"), false);
        assert_eq!(is_valid_name("."), false);
        assert_eq!(is_valid_name("**"), false);
    }

    #[test]
    fn point_light_radius_test() {
        let radius = point_light_radius(0.1, 0.2, 0.3, 0.001);
        assert_eq!(98.98999, radius);
    }

    #[test]
    fn is_sphere_visible_to_frustum_test() {
        let frustum = frustum_from_perspective(39.6_f32.to_radians(), 1280.0 / 720.0, 0.01, 1000.0);
        let sphere = Sphere3D::new(glam::vec3(-300.0, 0.0, 0.0), 5.0);
        assert_eq!(is_sphere_visible_to_frustum(&sphere, &frustum), false);

        let frustum = frustum_from_perspective(39.6_f32.to_radians(), 1280.0 / 720.0, 0.01, 1000.0);
        let sphere = Sphere3D::new(glam::vec3(0.0, 0.0, 0.0), 5.0);
        assert_eq!(is_sphere_visible_to_frustum(&sphere, &frustum), true);

        let frustum = frustum_from_perspective(39.6_f32.to_radians(), 1280.0 / 720.0, 0.01, 1000.0);
        let sphere = Sphere3D::new(glam::vec3(0.0, 0.0, 1005.0), 5.0);
        assert_eq!(is_sphere_visible_to_frustum(&sphere, &frustum), true);

        let frustum = frustum_from_perspective(39.6_f32.to_radians(), 1280.0 / 720.0, 0.01, 1000.0);
        let sphere = Sphere3D::new(glam::vec3(0.0, 0.0, -10.0), 5.0);
        assert_eq!(is_sphere_visible_to_frustum(&sphere, &frustum), false);
    }

    #[test]
    fn split_frustum_test() {
        let frustum = frustum_from_perspective(39.6_f32.to_radians(), 1280.0 / 720.0, 0.01, 1000.0);
        let clusters = split_frustum(&frustum, 9, 9, 9);
        assert_eq!(clusters.len(), 1000);
        for cluster in clusters.chunks(100) {
            let mut iter = cluster.chunks(10);
            if let Some(frustums) = iter.next() {
                assert!(frustums[0].near_3.eq(&frustum.near_3));
            }
            break;
        }
    }

    #[test]
    fn split_frustum_test2() {
        let frustum = frustum_from_perspective(39.6_f32.to_radians(), 1280.0 / 720.0, 0.01, 1000.0);
        let frustums = split_frustum(&frustum, 0, 0, 0);
        assert_eq!(frustums.len(), 1);
        assert!(frustums[0] == frustum, "{:?} == {:?}", frustums[0], frustum);
    }

    #[test]
    fn split_frustum_test3() {
        let frustum = frustum_from_perspective(39.6_f32.to_radians(), 1280.0 / 720.0, 0.01, 1000.0);
        let rotation = glam::Quat::from_euler(glam::EulerRot::XYZ, -0.0, 1.532398, -0.0);
        let transform = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::ONE,
            rotation,
            glam::Vec3::ZERO,
        );
        let new_frustum = frustum.transform(&transform);
        let frustums = split_frustum(&new_frustum, 0, 0, 0);
        assert_eq!(frustums.len(), 1);
        assert!(
            frustums[0] == new_frustum,
            "{:?} == {:?}",
            frustums[0],
            new_frustum
        );
    }

    #[test]
    fn subdivide_two_points_test() {
        let points =
            subdivide_two_points(9, &glam::vec3(0.0, 0.0, 0.0), &glam::vec3(10.0, 0.0, 0.0));
        assert_eq!(points.len(), 11);
        assert!(points[0].abs_diff_eq(glam::vec3(0.0, 0.0, 0.0), 0.001));
        assert!(points[9].abs_diff_eq(glam::vec3(9.0, 0.0, 0.0), 0.001));
        assert!(points[10].abs_diff_eq(glam::vec3(10.0, 0.0, 0.0), 0.001));
    }

    #[test]
    fn subdivide_four_points_test() {
        let points = subdivide_four_points(
            9,
            9,
            &glam::vec3(10.0, 0.0, 0.0),
            &glam::vec3(10.0, 0.0, -10.0),
            &glam::vec3(0.0, 0.0, -10.0),
            &glam::vec3(0.0, 0.0, 0.0),
        );
        assert_eq!(points.len(), 100);
        assert!(points[0].0.abs_diff_eq(glam::vec3(1.0, 0.0, 0.0), 0.001));
        assert!(points[0].1.abs_diff_eq(glam::vec3(1.0, 0.0, -1.0), 0.001));
        assert!(points[0].2.abs_diff_eq(glam::vec3(0.0, 0.0, -1.0), 0.001));
        assert!(points[0].3.abs_diff_eq(glam::vec3(0.0, 0.0, 0.0), 0.001));
    }

    #[test]
    fn generate_circle_points_test() {
        let points = generate_circle_points(glam::vec2(100.0, 100.0), 50.0, 8);
        assert_eq!(points[0], glam::vec2(150.0, 100.0));
    }

    #[test]
    fn is_point_in_polygon_test() {
        let polygon = vec![
            glam::Vec2::new(0.0, 0.0),
            glam::Vec2::new(0.0, 5.0),
            glam::Vec2::new(5.0, 5.0),
            glam::Vec2::new(5.0, 0.0),
        ];
        let test_points = vec![
            (glam::Vec2::new(2.0, 2.0), true),
            (glam::Vec2::new(6.0, 3.0), false),
            (glam::Vec2::new(0.0, 2.0), true),
            (glam::Vec2::new(3.0, 5.0), true),
            (glam::Vec2::new(2.5, 0.0), true),
            (glam::Vec2::new(5.0, 2.5), true),
        ];
        for (point, expected) in test_points {
            let result = is_point_in_polygon(point, &polygon, true);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn is_point_in_polygon_test1() {
        let polygon = vec![
            glam::Vec2::new(0.0, 0.0),
            glam::Vec2::new(0.0, 5.0),
            glam::Vec2::new(5.0, 5.0),
            glam::Vec2::new(5.0, 0.0),
        ];
        let test_points = vec![
            (glam::Vec2::new(2.0, 2.0), true),
            (glam::Vec2::new(6.0, 3.0), false),
            (glam::Vec2::new(0.0, 2.0), true),
            (glam::Vec2::new(3.0, 5.0), false),
            (glam::Vec2::new(2.5, 0.0), true),
            (glam::Vec2::new(5.0, 2.5), false),
        ];
        for (point, expected) in test_points {
            let result = is_point_in_polygon(point, &polygon, false);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn distance_from_point_to_segment_test() {
        assert_eq!(
            distance_from_point_to_segment(
                glam::vec2(0.0, 0.0),
                glam::vec2(5.0, 0.0),
                glam::vec2(1.0, 0.0)
            ),
            0.0
        );
        assert_eq!(
            distance_from_point_to_segment(
                glam::vec2(0.0, 0.0),
                glam::vec2(5.0, 0.0),
                glam::vec2(-1.0, 0.0)
            ),
            1.0
        );
        assert_eq!(
            distance_from_point_to_segment(
                glam::vec2(0.0, 0.0),
                glam::vec2(5.0, 0.0),
                glam::vec2(6.0, 0.0)
            ),
            1.0
        );
        assert_eq!(
            distance_from_point_to_segment(
                glam::vec2(0.0, 0.0),
                glam::vec2(5.0, 0.0),
                glam::vec2(6.0, 1.0)
            ),
            glam::vec2(5.0, 0.0).distance(glam::vec2(6.0, 1.0))
        );
        assert_eq!(
            distance_from_point_to_segment(
                glam::vec2(0.0, 0.0),
                glam::vec2(5.0, 0.0),
                glam::vec2(3.0, 1.0)
            ),
            1.0
        );
        assert_eq!(
            distance_from_point_to_segment(
                glam::vec2(0.0, 0.0),
                glam::vec2(5.0, 0.0),
                glam::vec2(-1.0, 1.0)
            ),
            glam::vec2(0.0, 0.0).distance(glam::vec2(-1.0, 1.0))
        );
    }

    #[test]
    fn get_sha256_from_reader_test() {
        let data: Vec<u8> = vec![1, 2, 3, 4];
        let sha256 = get_sha256_from_reader(&mut Cursor::new(data));
        assert_eq!(
            "9f64a767e1b97f131fabb6b467296c9b6f21e79fb3c5356e6c77e89b6a806a",
            sha256
        );
    }
}
