use crate::{
    content::{content_file_type::EContentFileType, level::LevelPhysics},
    engine::Engine,
    player_viewport::PlayerViewport,
};

pub trait Component {
    fn get_name(&self) -> String;

    fn set_name(&mut self, new_name: String);

    fn get_final_transformation(&self) -> glam::Mat4;

    fn set_transformation(&mut self, transformation: glam::Mat4);

    fn get_transformation(&self) -> glam::Mat4;

    fn on_post_update_transformation(
        &mut self,
        level_physics: Option<&mut crate::content::level::LevelPhysics>,
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

    fn initialize_physics(&mut self, level_physics: &mut LevelPhysics);

    fn tick(&mut self, time: f32, engine: &mut Engine, level_physics: &mut LevelPhysics);
}
