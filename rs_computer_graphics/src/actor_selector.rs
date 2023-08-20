use std::sync::Arc;

use crate::{actor::Actor, camera::Camera, util::ray_intersection_hit_test};

pub struct ActorSelector {}

impl ActorSelector {
    pub fn select<'a>(
        actors: Vec<&'a Actor>,
        mouse_position: winit::dpi::PhysicalPosition<f64>,
        window_size: winit::dpi::PhysicalSize<u32>,
        camera: &Camera,
    ) -> Option<(usize, &'a Actor)> {
        for (index, actor) in actors.iter().enumerate() {
            let hit_test_results = ray_intersection_hit_test(
                actor,
                mouse_position,
                window_size,
                *actor.get_model_matrix(),
                &camera,
            );

            if hit_test_results.is_empty() == false {
                return Some((index, actor));
            }
        }
        None
    }
}
