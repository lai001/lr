use core::ffi;
use glam::Vec4Swizzles;

pub fn ffi_to_rs_string(c_str: *const std::ffi::c_char) -> Option<String> {
    if c_str.is_null() {
        None
    } else {
        let rs_string = unsafe { ffi::CStr::from_ptr(c_str).to_str().unwrap().to_owned() };
        Some(rs_string)
    }
}

#[repr(C)]
#[derive(Clone, Default, PartialEq, Eq, Hash, Debug)]
pub struct Range<T: Copy> {
    pub start: T,
    pub end: T,
}

impl<T> Range<T>
where
    T: Copy,
{
    pub fn to_std_range(&self) -> std::ops::Range<T> {
        std::ops::Range::<T> {
            start: self.start,
            end: self.end,
        }
    }
}

pub fn math_remap_value_range(
    value: f64,
    from_range: std::ops::Range<f64>,
    to_range: std::ops::Range<f64>,
) -> f64 {
    (value - from_range.start) / (from_range.end - from_range.start)
        * (to_range.end - to_range.start)
        + to_range.start
}

pub fn screent_space_to_world_space(
    point: glam::Vec3,
    model: glam::Mat4,
    view: glam::Mat4,
    projection: glam::Mat4,
) -> glam::Vec3 {
    let point = glam::Vec4::new(point.x, point.y, point.z, 1.0);
    let mvp = projection * view * model;
    let inv_vp = mvp.inverse();
    let point_at_world_space = inv_vp * point;
    let point_at_world_space = point_at_world_space / point_at_world_space.w;
    point_at_world_space.xyz()
}

pub fn triangle_plane_normal_vector(a: glam::Vec3, b: glam::Vec3, c: glam::Vec3) -> glam::Vec3 {
    let u = b - a;
    let v = c - a;
    u.cross(v)
}

pub fn is_same_side(a: glam::Vec3, b: glam::Vec3, c: glam::Vec3, p: glam::Vec3) -> bool {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let v1 = ab.cross(ac);
    let v2 = ab.cross(ap);
    let result = v1.dot(v2) > 0.0;
    result
}

pub fn is_point_in_triangle(a: glam::Vec3, b: glam::Vec3, c: glam::Vec3, p: glam::Vec3) -> bool {
    is_same_side(a, b, c, p) && is_same_side(b, c, a, p) && is_same_side(c, a, b, p)
}

pub fn triangle_plane_ray_intersection(
    a: glam::Vec3,
    b: glam::Vec3,
    c: glam::Vec3,
    origin: glam::Vec3,
    direction: glam::Vec3,
) -> Option<glam::Vec3> {
    let direction = direction.normalize();
    let normal_vector = triangle_plane_normal_vector(a, b, c).normalize();
    let t = (a - origin).dot(normal_vector) / (direction.dot(normal_vector));

    let target_location = origin + direction * t;
    let is_point_in_triangle = is_point_in_triangle(a, b, c, target_location);
    if is_point_in_triangle {
        Some(target_location)
    } else {
        None
    }
}

pub fn shape(
    a: glam::Vec3,
    b: glam::Vec3,
    c: glam::Vec3,
    d: glam::Vec3,
) -> Vec<(glam::Vec3, glam::Vec3, glam::Vec3)> {
    let mut array: Vec<(glam::Vec3, glam::Vec3, glam::Vec3)> = vec![];
    array.push((a, b, c));
    array.push((a, c, d));
    array
}

pub fn change_working_directory() {
    if let (Ok(current_dir), Ok(current_exe)) = (std::env::current_dir(), std::env::current_exe()) {
        let current_exe_dir = std::path::Path::new(&current_exe)
            .parent()
            .unwrap()
            .to_str()
            .unwrap();
        let current_dir = current_dir.to_str().unwrap();
        if current_dir != current_exe_dir {
            std::env::set_current_dir(current_exe_dir).unwrap();
            log::info!("current_dir: {}", current_exe_dir);
        }
    }
}

pub fn get_object_address<T>(object: &T) -> String {
    let raw_ptr = object as *const T;
    std::format!("{:?}", raw_ptr)
}

pub fn cast_to_raw_buffer<'a, T>(vec: &[T]) -> &'a [u8] {
    let buffer = vec.as_ptr() as *const u8;
    let size = std::mem::size_of::<T>() * vec.len();
    let buffer = unsafe { std::slice::from_raw_parts(buffer, size) };
    buffer
}

pub fn cast_to_buffer<'a, U>(buffer: *const u8, len: usize) -> &'a [U] {
    unsafe {
        let len = len / std::mem::size_of::<U>();
        std::slice::from_raw_parts(buffer as *const U, len)
    }
}

#[cfg(test)]
pub mod test {
    use super::{math_remap_value_range, triangle_plane_ray_intersection};

    #[test]
    pub fn math_remap_value_range_test() {
        let mapped_value = math_remap_value_range(
            1.0,
            std::ops::Range::<f64> {
                start: 0.0,
                end: 2.0,
            },
            std::ops::Range::<f64> {
                start: 0.0,
                end: 100.0,
            },
        );
        assert_eq!(mapped_value, 50.0_f64);

        let mapped_value = math_remap_value_range(
            0.0,
            std::ops::Range::<f64> {
                start: 0.0,
                end: 2.0,
            },
            std::ops::Range::<f64> {
                start: 0.0,
                end: 100.0,
            },
        );
        assert_eq!(mapped_value, 0.0_f64);

        let mapped_value = math_remap_value_range(
            2.0,
            std::ops::Range::<f64> {
                start: 0.0,
                end: 2.0,
            },
            std::ops::Range::<f64> {
                start: 0.0,
                end: 100.0,
            },
        );
        assert_eq!(mapped_value, 100.0_f64);

        let mapped_value = math_remap_value_range(
            -1.0,
            std::ops::Range::<f64> {
                start: 0.0,
                end: 2.0,
            },
            std::ops::Range::<f64> {
                start: 0.0,
                end: 100.0,
            },
        );
        assert_eq!(mapped_value, -50.0_f64);
    }

    #[test]
    pub fn triangle_plane_ray_intersection_test() {
        let a = glam::Vec3 {
            x: -1.0,
            y: 1.0,
            z: 1.0,
        };
        let b = glam::Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        let c = glam::Vec3 {
            x: 0.0,
            y: -1.0,
            z: 1.0,
        };
        let origin = glam::Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let direction = glam::Vec3 {
            x: 0.0,
            y: -1.0,
            z: 1.0,
        } - origin;
        let intersection_point = triangle_plane_ray_intersection(a, b, c, origin, direction);
        println!("{:?}", intersection_point);
    }
}
