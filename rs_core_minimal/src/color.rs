pub const BLACK: glam::Vec4 = glam::vec4(0.0, 0.0, 0.0, 1.0);
pub const RED: glam::Vec4 = glam::vec4(1.0, 0.0, 0.0, 1.0);
pub const GREEN: glam::Vec4 = glam::vec4(0.0, 1.0, 0.0, 1.0);
pub const BLUE: glam::Vec4 = glam::vec4(0.0, 0.0, 1.0, 1.0);
pub const WHITE: glam::Vec4 = glam::vec4(1.0, 1.0, 1.0, 1.0);
pub const TRANSPARENT: glam::Vec4 = glam::vec4(0.0, 0.0, 0.0, 0.0);

pub fn random_color3() -> glam::Vec3 {
    let x: f32 = rand::Rng::random_range(&mut rand::rng(), 0.0..1.0);
    let y: f32 = rand::Rng::random_range(&mut rand::rng(), 0.0..1.0);
    let z: f32 = rand::Rng::random_range(&mut rand::rng(), 0.0..1.0);
    glam::vec3(x, y, z)
}

pub fn random_color4() -> glam::Vec4 {
    let x: f32 = rand::Rng::random_range(&mut rand::rng(), 0.0..1.0);
    let y: f32 = rand::Rng::random_range(&mut rand::rng(), 0.0..1.0);
    let z: f32 = rand::Rng::random_range(&mut rand::rng(), 0.0..1.0);
    let w: f32 = rand::Rng::random_range(&mut rand::rng(), 0.0..1.0);
    glam::vec4(x, y, z, w)
}
