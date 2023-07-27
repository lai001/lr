use crate::{model_loader::ModelLoader, rotator::Rotator, static_mesh::StaticMesh};

pub struct Actor {
    static_meshs: Vec<StaticMesh>,
    model: glam::Mat4,
    localtion: glam::Vec3,
    rotator: Rotator,
}

impl Actor {
    pub fn load_from_file(device: &wgpu::Device, queue: &wgpu::Queue, file_path: &str) -> Actor {
        let static_meshs = ModelLoader::load_from_file(device, queue, file_path);
        let actor = Actor {
            static_meshs,
            model: glam::Mat4::IDENTITY,
            localtion: glam::Vec3::ZERO,
            rotator: Rotator::zero(),
        };
        actor
    }

    pub fn set_world_location(&mut self, location: glam::Vec3) {
        self.localtion = location;
        self.model = glam::Mat4::from_translation(self.localtion)
            * self.rotator.to_radians().to_matrix()
            * glam::Mat4::IDENTITY;
    }

    pub fn set_rotator(&mut self, rotator: Rotator) {
        self.rotator = rotator;
        self.model = glam::Mat4::from_translation(self.localtion)
            * self.rotator.to_radians().to_matrix()
            * glam::Mat4::IDENTITY;
    }

    pub fn get_localtion(&self) -> glam::Vec3 {
        self.localtion
    }

    pub fn get_rotator(&self) -> Rotator {
        self.rotator
    }

    pub fn get_model_matrix(&self) -> &glam::Mat4 {
        &self.model
    }

    pub fn get_static_meshs(&self) -> &[StaticMesh] {
        self.static_meshs.as_ref()
    }

    pub fn get_static_meshs_mut(&mut self) -> &mut [StaticMesh] {
        &mut self.static_meshs
    }

    pub fn set_model_matrix(&mut self, model_matrix: glam::Mat4) {
        self.model = model_matrix;
    }
}
