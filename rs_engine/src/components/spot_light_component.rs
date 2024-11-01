use super::point_light_component::PointLight;
use crate::scene_node::{EComponentType, SceneNode};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct SpotLight {
    pub light: PointLight,
    pub cut_off: f32,
    pub outer_cut_off: f32,
}

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            light: PointLight::default(),
            cut_off: 12.5_f32.to_radians(),
            outer_cut_off: 17.5_f32.to_radians(),
        }
    }
}

#[derive(Clone)]
pub struct SpotLightComponentRuntime {
    pub parent_final_transformation: glam::Mat4,
    pub final_transformation: glam::Mat4,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SpotLightComponent {
    pub name: String,
    pub transformation: glam::Mat4,
    pub is_visible: bool,
    pub spot_light: SpotLight,

    #[serde(skip)]
    pub run_time: Option<SpotLightComponentRuntime>,
}
impl SpotLightComponent {
    pub fn new(name: String, transformation: glam::Mat4) -> Self {
        Self {
            name,
            transformation,
            is_visible: true,
            run_time: None,
            spot_light: SpotLight::default(),
        }
    }

    pub fn new_scene_node(
        name: String,
        transformation: glam::Mat4,
    ) -> SingleThreadMutType<SceneNode> {
        let component = Self::new(name, transformation);
        let component = SingleThreadMut::new(component);
        let scene_node = SceneNode {
            component: EComponentType::SpotLightComponent(component),
            childs: vec![],
        };
        let scene_node = SingleThreadMut::new(scene_node);
        scene_node
    }
}
impl super::component::Component for SpotLightComponent {
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
        self.run_time = Some(SpotLightComponentRuntime {
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
