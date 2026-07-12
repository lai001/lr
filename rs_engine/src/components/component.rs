use crate::{
    content::{content_file_type::EContentFileType, level::LevelPhysics},
    drawable::Drawable,
    engine::Engine,
    player_viewport::PlayerViewport,
};
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::clone_trait_object;
use rapier3d::{dynamics::RigidBodyHandle, geometry::ColliderHandle};

#[typetag::serde]
pub trait Component: erased_serde::Serialize + Downcast + dyn_clone::DynClone {
    fn get_name(&self) -> String;

    fn set_name(&mut self, new_name: String);

    fn get_final_transformation(&self) -> glam::Mat4;

    fn set_transformation(&mut self, transformation: glam::Mat4);

    fn get_transformation(&self) -> glam::Mat4;

    fn on_post_update_transformation(
        &mut self,
        engine: &mut Engine,
        level_physics: Option<&mut LevelPhysics>,
        files: &[EContentFileType],
    );

    fn set_final_transformation(&mut self, final_transformation: glam::Mat4);

    fn set_parent_final_transformation(&mut self, parent_final_transformation: glam::Mat4);

    fn get_parent_final_transformation(&self) -> glam::Mat4;

    fn initialize(
        &mut self,
        engine: &mut Engine,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    );

    fn initialize_physics(
        &mut self,
        engine: &mut Engine,
        level_physics: &mut LevelPhysics,
        files: &[EContentFileType],
    );

    fn tick(&mut self, time: f32, engine: &mut Engine, level_physics: &mut LevelPhysics);

    fn collider_handles(&self) -> Vec<ColliderHandle> {
        vec![]
    }

    fn rigid_body_handle(&self) -> Option<RigidBodyHandle> {
        None
    }

    fn as_drawable(&self) -> Option<&dyn Drawable> {
        None
    }

    fn gizmo_default(&mut self, gizmo_final_transformation: Option<glam::Mat4>) {
        if let Some(gizmo_final_transformation) = gizmo_final_transformation {
            let parent_final_transformation = self.get_parent_final_transformation();
            self.set_transformation(
                parent_final_transformation.inverse() * gizmo_final_transformation,
            );
        }
    }

    fn gizmo(&mut self, gizmo_final_transformation: Option<glam::Mat4>) {
        self.gizmo_default(gizmo_final_transformation);
    }
}

impl_downcast!(Component);
clone_trait_object!(Component);
