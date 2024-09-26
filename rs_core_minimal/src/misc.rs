use crate::frustum::Frustum;

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
    let manifest = std::env::current_exe()
        .map(|x| x.join("../../../Cargo.toml"))
        .expect("Should be a valid path");
    let is_exists = manifest.exists();
    // let is_cargo_exist = get_engine_root_dir().join(".cargo").exists();
    // let is_xmake_exist = get_engine_root_dir().join(".xmake").exists();
    // let is_vscode_exist = get_engine_root_dir().join(".vscode").exists();
    // is_run_from_ide() || is_cargo_exist || is_xmake_exist || is_vscode_exist
    let vars = std::env::vars().filter(|x| x.0 == "CARGO_MANIFEST_DIR".to_string());
    vars.count() != 0 || is_exists
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
