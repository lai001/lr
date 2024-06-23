use crate::{build_content_file_url, url_extension::UrlExtension};
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use rs_foundation::new::SingleThreadMutType;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Serialize, Deserialize, Debug)]
pub struct DirectionalLight {
    eye: glam::Vec3,
    light_projection: glam::Mat4,
    light_view: glam::Mat4,
    transformation: glam::Mat4,
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
}

impl DirectionalLight {
    pub fn get_interactive_transformation(&mut self) -> &mut glam::Mat4 {
        &mut self.transformation
    }

    pub fn new(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
        eye: glam::Vec3,
    ) -> DirectionalLight {
        let light_projection = glam::Mat4::orthographic_rh(left, right, bottom, top, near, far);
        let up = glam::Vec3::new(0.0, 1.0, 0.0);
        let dir = glam::Vec3::ZERO - eye;
        let light_view = glam::Mat4::look_to_rh(eye, dir, up);

        let transformation = glam::Mat4::from_rotation_translation(
            glam::Quat::from_euler(glam::EulerRot::XYZ, dir.x, dir.y, dir.z),
            eye,
        );

        DirectionalLight {
            light_projection,
            left,
            right,
            bottom,
            top,
            near,
            far,
            light_view,
            eye,
            transformation,
        }
    }

    pub fn update_clip(&mut self, near: f32, far: f32) {
        self.near = near;
        self.far = far;
        self.update();
    }

    pub fn update_view_rect(&mut self, left: f32, right: f32, bottom: f32, top: f32) {
        self.left = left;
        self.right = right;
        self.bottom = bottom;
        self.top = top;
        self.update();
    }

    fn update(&mut self) {
        self.light_projection = glam::Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        );
    }

    pub fn get_light_projection(&self) -> &glam::Mat4 {
        &self.light_projection
    }

    pub fn get_light_view(&mut self) -> &mut glam::Mat4 {
        &mut self.light_view
    }

    pub fn get_light_space_matrix(&mut self) -> glam::Mat4 {
        let (_, rotation_quat, translation) = self.transformation.to_scale_rotation_translation();
        let (x, y, z) = rotation_quat.to_euler(glam::EulerRot::XYZ);
        let rotation = glam::Vec3::from_array([x, y, z]);
        let up = glam::Vec3::new(0.0, 1.0, 0.0);
        let direction = rotation;
        self.light_view = glam::Mat4::look_to_rh(translation, direction, up);
        self.light_projection * self.light_view
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Level {
    pub url: url::Url,
    pub actors: Vec<Rc<RefCell<crate::actor::Actor>>>,
    pub directional_lights: Vec<SingleThreadMutType<DirectionalLight>>,
}

impl Asset for Level {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(rs_artifact::content_type::EContentType::Level)
    }
}

impl Level {
    pub fn empty_level() -> Self {
        Self {
            actors: vec![],
            url: build_content_file_url("Empty").unwrap(),
            directional_lights: vec![],
        }
    }

    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }
}
