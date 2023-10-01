use rand::Rng;

pub const BLACK: glam::Vec4 = glam::vec4(0.0, 0.0, 0.0, 1.0);
pub const RED: glam::Vec4 = glam::vec4(1.0, 0.0, 0.0, 1.0);
pub const GREEN: glam::Vec4 = glam::vec4(0.0, 1.0, 0.0, 1.0);
pub const BLUE: glam::Vec4 = glam::vec4(0.0, 0.0, 1.0, 1.0);
pub const WHITE: glam::Vec4 = glam::vec4(1.0, 1.0, 1.0, 1.0);
pub const TRANSPARENT: glam::Vec4 = glam::vec4(0.0, 0.0, 0.0, 0.0);

pub fn random() -> glam::Vec4 {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(0.0_f32..1.0);
    let y = rng.gen_range(0.0_f32..1.0);
    let z = rng.gen_range(0.0_f32..1.0);
    glam::vec4(x, y, z, 1.0)
}
