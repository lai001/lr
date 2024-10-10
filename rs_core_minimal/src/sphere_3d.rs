pub struct Sphere3D {
    pub center: glam::Vec3,
    pub radius: f32,
}

impl Sphere3D {
    pub fn new(center: glam::Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
}
