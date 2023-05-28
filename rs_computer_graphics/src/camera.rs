use glam::{Vec3Swizzles, Vec4Swizzles};

pub struct Camera {
    pub(crate) world_location: glam::Vec3,
    pub(crate) center: glam::Vec3,
    up: glam::Vec3,
    forward_vector: glam::Vec3,
    pub(crate) fov_y_radians: f32,
    pub(crate) aspect_ratio: f32,
    pub(crate) z_near: f32,
    pub(crate) z_far: f32,
    pub(crate) projection_matrix: glam::Mat4,
    pub(crate) view_matrix: glam::Mat4,
    pub(crate) view_projection: glam::Mat4,
}

impl Camera {
    pub fn new(
        world_location: glam::Vec3,
        center: glam::Vec3,
        up: glam::Vec3,
        fov_y_radians: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32,
    ) -> Camera {
        let projection_matrix =
            glam::Mat4::perspective_rh(fov_y_radians, aspect_ratio, z_near, z_far);
        let view_matrix = glam::Mat4::look_at_rh(world_location, center, glam::Vec3::Y);
        Camera {
            world_location,
            fov_y_radians,
            aspect_ratio,
            z_near,
            z_far,
            projection_matrix,
            view_matrix,
            center,
            up,
            forward_vector: (center - world_location).normalize(),
            view_projection: projection_matrix * view_matrix,
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

    pub fn set_world_absolute_location(&mut self, world_location: glam::Vec3) {
        self.world_location = world_location;
        self.center = self.world_location + self.forward_vector;

        self.update_view_matrix();
    }

    pub fn add_world_absolute_location(&mut self, world_location: glam::Vec3) {
        self.world_location = self.world_location + world_location;
        self.center = self.world_location + self.forward_vector;

        self.update_view_matrix();
    }

    pub fn add_rotation(&mut self, axis: glam::Vec3, angle_degrees: f32) {
        let matrix = glam::Mat4::from_quat(glam::Quat::from_axis_angle(
            axis,
            angle_degrees.to_radians(),
        ));
        let mut forward_vector = self.forward_vector.xyzx();
        forward_vector.w = 0.0;
        self.forward_vector = (matrix * forward_vector).xyz().normalize();
        self.center = self.world_location + self.forward_vector;
        self.update_view_matrix();
    }

    fn update_projection_matrix(&mut self) {
        self.projection_matrix = glam::Mat4::perspective_rh(
            self.fov_y_radians,
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        );
        self.update_vp();
    }

    fn update_view_matrix(&mut self) {
        self.view_matrix = glam::Mat4::look_at_rh(self.world_location, self.center, glam::Vec3::Y);
        self.update_vp();
    }

    fn update_vp(&mut self) {
        self.view_projection = self.projection_matrix * self.view_matrix;
    }

    pub fn get_view_matrix(&self) -> glam::Mat4 {
        return self.view_matrix;
    }

    pub fn get_projection_matrix(&self) -> glam::Mat4 {
        return self.projection_matrix;
    }

    pub fn get_view_projection(&self) -> glam::Mat4 {
        return self.view_projection;
    }
}
