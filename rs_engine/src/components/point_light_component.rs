use crate::scene_node::{EComponentType, SceneNode};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct PointLight {
    pub ambient: glam::Vec3,
    pub diffuse: glam::Vec3,
    pub specular: glam::Vec3,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            ambient: glam::Vec3::ONE,
            diffuse: glam::Vec3::ONE,
            specular: glam::Vec3::ONE,
            constant: 1.0,
            linear: 0.09,
            quadratic: 0.032,
        }
    }
}

#[derive(Clone)]
pub struct PointLightComponentRuntime {
    pub parent_final_transformation: glam::Mat4,
    pub final_transformation: glam::Mat4,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PointLightComponent {
    pub name: String,
    pub transformation: glam::Mat4,
    pub is_visible: bool,
    pub point_light: PointLight,
    #[serde(skip)]
    pub run_time: Option<PointLightComponentRuntime>,
}

impl PointLightComponent {
    pub fn new(name: String, transformation: glam::Mat4) -> Self {
        Self {
            name,
            transformation,
            is_visible: true,
            run_time: None,
            point_light: PointLight::default(),
        }
    }

    pub fn new_scene_node(
        name: String,
        transformation: glam::Mat4,
    ) -> SingleThreadMutType<SceneNode> {
        let component = Self::new(name, transformation);
        let component = SingleThreadMut::new(component);
        let scene_node = SceneNode {
            component: EComponentType::PointLightComponent(component),
            childs: vec![],
        };
        let scene_node = SingleThreadMut::new(scene_node);
        scene_node
    }
}

impl super::component::Component for PointLightComponent {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    fn get_final_transformation(&self) -> glam::Mat4 {
        let Some(run_time) = self.run_time.as_ref() else {
            return glam::Mat4::IDENTITY;
        };
        run_time.final_transformation
    }

    fn set_transformation(&mut self, transformation: glam::Mat4) {
        self.transformation = transformation;
    }

    fn get_transformation(&self) -> glam::Mat4 {
        self.transformation
    }

    fn on_post_update_transformation(
        &mut self,
        level_physics: Option<&mut crate::content::level::Physics>,
    ) {
        let _ = level_physics;
    }

    fn set_final_transformation(&mut self, final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.final_transformation = final_transformation;
    }

    fn set_parent_final_transformation(&mut self, parent_final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.parent_final_transformation = parent_final_transformation;
    }

    fn get_parent_final_transformation(&self) -> glam::Mat4 {
        let Some(run_time) = self.run_time.as_ref() else {
            return glam::Mat4::IDENTITY;
        };
        run_time.parent_final_transformation
    }

    fn initialize(
        &mut self,
        engine: &mut crate::engine::Engine,
        files: &[crate::content::content_file_type::EContentFileType],
        player_viewport: &mut crate::player_viewport::PlayerViewport,
    ) {
        let _ = player_viewport;
        let _ = files;
        let _ = engine;
        self.run_time = Some(PointLightComponentRuntime {
            parent_final_transformation: glam::Mat4::IDENTITY,
            final_transformation: glam::Mat4::IDENTITY,
        })
    }

    fn initialize_physics(
        &mut self,
        rigid_body_set: &mut rapier3d::prelude::RigidBodySet,
        collider_set: &mut rapier3d::prelude::ColliderSet,
    ) {
        let _ = collider_set;
        let _ = rigid_body_set;
    }

    fn tick(
        &mut self,
        time: f32,
        engine: &mut crate::engine::Engine,
        rigid_body_set: &mut rapier3d::prelude::RigidBodySet,
        collider_set: &mut rapier3d::prelude::ColliderSet,
    ) {
        let _ = collider_set;
        let _ = rigid_body_set;
        let _ = engine;
        let _ = time;
    }
}
