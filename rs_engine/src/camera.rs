use crate::{misc::FORWARD_VECTOR, rotator::Rotator};
use rs_core_minimal::frustum::Frustum;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveProperties {
    pub fov_y_radians: f32,
    pub aspect_ratio: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrthographicProperties {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

impl OrthographicProperties {
    pub fn from_scale(scale: f32) -> OrthographicProperties {
        OrthographicProperties {
            left: scale,
            right: scale,
            bottom: scale,
            top: scale,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ECameraType {
    Perspective(PerspectiveProperties),
    Orthographic(OrthographicProperties),
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    world_location: glam::Vec3,
    up_vector: glam::Vec3,
    forward_vector: glam::Vec3,
    z_near: f32,
    z_far: f32,
    projection_matrix: glam::Mat4,
    view_matrix: glam::Mat4,
    rotator: Rotator,
    camera_type: ECameraType,
}

impl Camera {
    pub fn default_forward_vector() -> glam::Vec3 {
        FORWARD_VECTOR
    }

    fn new(
        world_location: glam::Vec3,
        forward_vector: glam::Vec3,
        up_vector: glam::Vec3,
        z_near: f32,
        z_far: f32,
        camera_type: ECameraType,
    ) -> Camera {
        let projection_matrix = match camera_type {
            ECameraType::Perspective(perspective_properties) => glam::Mat4::perspective_rh(
                perspective_properties.fov_y_radians,
                perspective_properties.aspect_ratio,
                z_near,
                z_far,
            ),
            ECameraType::Orthographic(orthographic_properties) => glam::Mat4::orthographic_rh(
                orthographic_properties.left,
                orthographic_properties.right,
                orthographic_properties.bottom,
                orthographic_properties.top,
                z_near,
                z_far,
            ),
        };

        let view_matrix = glam::Mat4::look_to_rh(world_location, forward_vector, up_vector);
        Camera {
            world_location,
            up_vector,
            forward_vector,
            z_near,
            z_far,
            projection_matrix,
            view_matrix,
            rotator: Rotator::from_forward_vector(forward_vector),
            camera_type,
        }
    }

    pub fn default(window_width: u32, window_height: u32) -> Camera {
        Self::new(
            glam::Vec3::new(0.0, 0.0, 0.0),
            Self::default_forward_vector(),
            glam::Vec3::new(0.0, 1.0, 0.0),
            0.1,
            1000.0,
            ECameraType::Perspective(PerspectiveProperties {
                fov_y_radians: 39.6_f32.to_radians(),
                aspect_ratio: window_width as f32 / window_height as f32,
            }),
        )
    }

    pub fn set_window_size(&mut self, window_width: u32, window_height: u32) {
        match &mut self.camera_type {
            ECameraType::Perspective(perspective_properties) => {
                perspective_properties.aspect_ratio = window_width as f32 / window_height as f32;
            }
            ECameraType::Orthographic(_) => {}
        }
        self.update_projection_matrix();
    }

    pub fn set_fov_y_radians(&mut self, fov_y_radians: f32) {
        match &mut self.camera_type {
            ECameraType::Perspective(perspective_properties) => {
                perspective_properties.fov_y_radians = fov_y_radians;
            }
            ECameraType::Orthographic(_) => {}
        }
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
        // let mut forward_vector = glam::Vec3::ZERO;
        // let pitch = rotator
        //     .pitch
        //     .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        // forward_vector.x = pitch.cos() * rotator.yaw.cos();
        // forward_vector.y = pitch.sin();
        // forward_vector.z = pitch.cos() * rotator.yaw.sin();
        self.forward_vector = rotator.to_forward_vector();
        self.update_view_matrix();
    }

    pub fn add_world_rotation_relative(&mut self, rotator: &Rotator) {
        self.rotator.pitch = (self.rotator.pitch - rotator.pitch)
            .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        self.rotator.yaw -= rotator.yaw;
        self.rotator.roll += rotator.roll;
        self.set_world_rotation_absolute(&self.rotator.clone());
    }

    fn update_projection_matrix(&mut self) {
        match self.camera_type {
            ECameraType::Perspective(perspective_properties) => {
                self.projection_matrix = glam::Mat4::perspective_rh(
                    perspective_properties.fov_y_radians,
                    perspective_properties.aspect_ratio,
                    self.z_near,
                    self.z_far,
                );
            }
            ECameraType::Orthographic(orthographic_properties) => {
                self.projection_matrix = glam::Mat4::orthographic_rh(
                    orthographic_properties.left,
                    orthographic_properties.right,
                    orthographic_properties.bottom,
                    orthographic_properties.top,
                    self.z_near,
                    self.z_far,
                );
            }
        }
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

    pub fn get_view_projection_matrix(&self) -> glam::Mat4 {
        return self.projection_matrix * self.view_matrix;
    }

    pub fn get_z_far(&self) -> f32 {
        self.z_far
    }

    pub fn get_z_near(&self) -> f32 {
        self.z_near
    }

    pub fn get_camera_type(&self) -> ECameraType {
        self.camera_type
    }

    pub fn set_forward_vector(&mut self, forward_vector: glam::Vec3) {
        self.forward_vector = forward_vector;
        self.update_view_matrix();
    }

    pub fn get_right_vector(&self) -> glam::Vec3 {
        self.forward_vector.cross(self.up_vector)
    }

    pub fn get_world_transformation(&self) -> glam::Mat4 {
        glam::Mat4::from_translation(self.world_location) * self.rotator.to_matrix()
    }

    pub fn get_frustum_no_apply_tramsformation(&self) -> Frustum {
        match self.camera_type {
            ECameraType::Perspective(perspective_properties) => {
                let frustum = rs_core_minimal::misc::frustum_from_perspective(
                    perspective_properties.fov_y_radians,
                    perspective_properties.aspect_ratio,
                    self.z_near,
                    self.z_far,
                );
                frustum
            }
            ECameraType::Orthographic(_) => unimplemented!(),
        }
    }

    pub fn get_frustum_apply_custom_tramsformation(&self, transform: &glam::Mat4) -> Frustum {
        let frustum = self.get_frustum_no_apply_tramsformation();
        frustum.transform(transform)
    }

    pub fn get_frustum_apply_tramsformation(&self) -> Frustum {
        let transform: &glam::Mat4 = &self.get_world_transformation();
        self.get_frustum_apply_custom_tramsformation(transform)
    }

    pub fn get_render_frustum_apply_tramsformation(
        &self,
    ) -> rs_render::global_uniform::CameraFrustum {
        let frustum = self.get_frustum_apply_tramsformation();
        let mut render_frustum = rs_render::global_uniform::CameraFrustum::default();
        render_frustum.near_0 = frustum.near_0;
        render_frustum.near_1 = frustum.near_1;
        render_frustum.near_2 = frustum.near_2;
        render_frustum.near_3 = frustum.near_3;
        render_frustum.far_0 = frustum.far_0;
        render_frustum.far_1 = frustum.far_1;
        render_frustum.far_2 = frustum.far_2;
        render_frustum.far_3 = frustum.far_3;
        render_frustum
    }
}
