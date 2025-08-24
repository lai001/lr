use crate::{
    egui_state::update_viewport_info,
    motion_event::{EActionType, Geometry, MotionEvent},
};
use rs_render::egui_render::EGUIRenderOutput;
use std::collections::VecDeque;
use winit::{dpi::PhysicalPosition, event::DeviceId};

pub struct GUI {
    window_size: winit::dpi::PhysicalSize<u32>,
    scale_factor: f32,
    egui_context: egui::Context,
    geometries: VecDeque<Geometry>,
    status_bar_height: i32,
    state: crate::egui_state::State,
}

impl GUI {
    pub fn new(scale_factor: f32, width: u32, height: u32, status_bar_height: i32) -> Self {
        let egui_context = egui::Context::default();
        let state = crate::egui_state::State::new(
            egui_context.clone(),
            egui::ViewportId::ROOT,
            Some(scale_factor),
            None,
            None,
        );
        let window_size = winit::dpi::PhysicalSize::<u32>::new(width, height);
        GUI {
            scale_factor,
            egui_context,
            geometries: VecDeque::new(),
            status_bar_height,
            state,
            window_size,
        }
    }

    pub fn on_size_changed(&mut self, width: u32, height: u32) {
        let _ = self.state.on_window_event(
            &winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::<u32>::new(
                width, height,
            )),
            self.scale_factor,
        );
    }

    pub fn on_touch(&mut self, mut motion_event: MotionEvent<'_>) {
        let new_geometry = motion_event.to_geometry();
        let phase: winit::event::TouchPhase = {
            match new_geometry.action {
                EActionType::ActionUp => winit::event::TouchPhase::Ended,
                EActionType::ActionMove => winit::event::TouchPhase::Moved,
                EActionType::ActionDown => winit::event::TouchPhase::Started,
                EActionType::ActionCancel => winit::event::TouchPhase::Cancelled,
                EActionType::ActionOutside => winit::event::TouchPhase::Ended,
            }
        };

        let event = winit::event::WindowEvent::Touch(winit::event::Touch {
            device_id: DeviceId::dummy(),
            phase,
            location: PhysicalPosition::<f64>::new(new_geometry.x as f64, new_geometry.y as f64),
            force: None,
            id: 0,
        });
        let _ = self.state.on_window_event(&event, self.scale_factor);
        self.geometries.push_front(new_geometry);
        if self.geometries.len() > 2 {
            self.geometries.drain(0..1);
        }
    }

    pub fn get_delta(&self) -> glam::Vec2 {
        if let (Some(latest), Some(previous)) = (self.geometries.get(0), self.geometries.get(1)) {
            return glam::vec2(latest.x - previous.x, latest.y - previous.y);
        }
        glam::Vec2::ZERO
    }

    pub fn get_position(&self) -> Option<glam::Vec2> {
        if let Some(latest) = self.geometries.get(0) {
            return Some(glam::vec2(latest.x, latest.y));
        }
        return None;
    }

    pub fn get_action(&self) -> Option<EActionType> {
        if let Some(latest) = self.geometries.get(0) {
            return Some(latest.action.clone());
        }
        None
    }

    pub fn begin_ui(&mut self) {
        let egui_ctx = &self.egui_context;
        let viewport_id = self.state.egui_input().viewport_id;
        let viewport_info: &mut egui::ViewportInfo = self
            .state
            .egui_input_mut()
            .viewports
            .get_mut(&viewport_id)
            .unwrap();

        update_viewport_info(
            viewport_info,
            egui_ctx,
            false,
            self.scale_factor,
            Some(false),
            true,
            PhysicalPosition::new(0, 0),
            self.window_size,
            PhysicalPosition::new(0, 0),
            self.window_size,
            Some(format!("LR")),
            None,
            None,
            true,
        );
        let outer_size: winit::dpi::PhysicalSize<u32> = self.window_size;
        let inner_size: winit::dpi::PhysicalSize<u32> = self.window_size;
        let input = self
            .state
            .take_egui_input(outer_size, inner_size, self.scale_factor);
        egui_ctx.begin_pass(input);
        egui_ctx.clear_animations();
    }

    pub fn end_ui(&mut self, window_id: isize) -> EGUIRenderOutput {
        let context = &self.egui_context;
        let full_output = context.end_pass();
        let gui_render_output = rs_render::egui_render::EGUIRenderOutput {
            textures_delta: full_output.textures_delta,
            clipped_primitives: context
                .tessellate(full_output.shapes, full_output.pixels_per_point),
            window_id,
        };
        gui_render_output
    }

    pub fn set_status_bar_height(&mut self, status_bar_height: i32) {
        self.status_bar_height = status_bar_height;
    }

    pub fn egui_context(&self) -> &egui::Context {
        &self.egui_context
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
}
