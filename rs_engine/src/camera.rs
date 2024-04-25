use crate::rotator::Rotator;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    world_location: glam::Vec3,
    up_vector: glam::Vec3,
    forward_vector: glam::Vec3,
    fov_y_radians: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
    projection_matrix: glam::Mat4,
    view_matrix: glam::Mat4,
    rotator: Rotator,
}

impl Camera {
    fn new(
        world_location: glam::Vec3,
        forward_vector: glam::Vec3,
        up_vector: glam::Vec3,
        fov_y_radians: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32,
    ) -> Camera {
        let projection_matrix =
            glam::Mat4::perspective_rh(fov_y_radians, aspect_ratio, z_near, z_far);
        let view_matrix = glam::Mat4::look_to_rh(world_location, forward_vector, up_vector);
        Camera {
            world_location,
            up_vector,
            forward_vector,
            fov_y_radians,
            aspect_ratio,
            z_near,
            z_far,
            projection_matrix,
            view_matrix,
            rotator: Rotator {
                yaw: 0.0,
                roll: 0.0,
                pitch: 0.0,
            },
        }
    }

    pub fn default(window_width: u32, window_height: u32) -> Camera {
        Self::new(
            glam::Vec3::new(0.0, 0.0, 0.0),
            glam::Vec3::new(0.0, 0.0, -1.0),
            glam::Vec3::new(0.0, 1.0, 0.0),
            39.6_f32.to_radians(),
            window_width as f32 / window_height as f32,
            0.01,
            1000.0,
        )
    }

    pub fn set_window_size(&mut self, window_width: u32, window_height: u32) {
        self.aspect_ratio = window_width as f32 / window_height as f32;
        self.update_projection_matrix();
    }

    pub fn set_fov_y_radians(&mut self, fov_y_radians: f32) {
        self.fov_y_radians = fov_y_radians;
        self.update_projection_matrix();
    }

    pub fn set_z_near(&mut self, z_near: f32) {
        self.z_near = z_near;
        self.update_projection_matrix();
    }

    pub fn set_z_far(&mut self, z_far: f32) {
        self.z_far = z_far;
        self.update_projection_matrix();
    }

    pub fn add_world_location(&mut self, world_location: glam::Vec3) {
        self.world_location = self.world_location + world_location;
        self.update_view_matrix();
    }

    pub fn set_world_location(&mut self, world_location: glam::Vec3) {
        self.world_location = world_location;
        self.update_view_matrix();
    }

    pub fn add_local_location(&mut self, location: glam::Vec3) {
        self.world_location += self.forward_vector * location.z;
        self.world_location += self.forward_vector.cross(self.up_vector).normalize() * location.x;
        self.world_location += self.up_vector * location.y;
        self.update_view_matrix();
    }

    pub fn set_world_rotation_absolute(&mut self, rotator: &Rotator) {
        let mut forward_vector = glam::Vec3::ZERO;
        let pitch = rotator
            .pitch
            .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        forward_vector.x = pitch.cos() * rotator.yaw.cos();
        forward_vector.y = pitch.sin();
        forward_vector.z = pitch.cos() * rotator.yaw.sin();
        self.forward_vector = forward_vector;
        self.update_view_matrix();
    }

    pub fn add_world_rotation_relative(&mut self, rotator: &Rotator) {
        self.rotator.pitch = (self.rotator.pitch + rotator.pitch)
            .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        self.rotator.yaw += rotator.yaw;
        self.rotator.roll += rotator.roll;
        self.set_world_rotation_absolute(&self.rotator.clone());
    }

    fn update_projection_matrix(&mut self) {
        self.projection_matrix = glam::Mat4::perspective_rh(
            self.fov_y_radians,
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        );
    }

    fn update_view_matrix(&mut self) {
        self.view_matrix =
            glam::Mat4::look_to_rh(self.world_location, self.forward_vector, self.up_vector);
    }

    pub fn get_view_matrix(&self) -> glam::Mat4 {
        return self.view_matrix;
    }

    pub fn get_projection_matrix(&self) -> glam::Mat4 {
        return self.projection_matrix;
    }

    pub fn get_world_location(&self) -> glam::Vec3 {
        self.world_location
    }

    pub fn get_forward_vector(&self) -> glam::Vec3 {
        self.forward_vector
    }
}
