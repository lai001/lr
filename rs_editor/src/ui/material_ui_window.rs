use super::material_view::MaterialView;
use crate::{
    custom_event::ECustomEventType, editor_context::EWindowType, windows_manager::WindowsManager,
};
use anyhow::anyhow;
use egui_winit::State;
use rs_engine::engine::Engine;
use rs_foundation::new::SingleThreadMutType;
use winit::event::WindowEvent;

pub struct DataSource {
    pub current_open_material: Option<SingleThreadMutType<crate::material::Material>>,
    pub is_shader_code_window_open: bool,
}

pub struct MaterialUIWindow {
    pub egui_winit_state: State,
    pub material_view: MaterialView,
    pub data_source: DataSource,
    pub context: egui::Context,
}

impl MaterialUIWindow {
    pub fn new(
        context: egui::Context,
        window_manager: &mut WindowsManager,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
        engine: &mut Engine,
    ) -> anyhow::Result<MaterialUIWindow> {
        let window_context =
            window_manager.spwan_new_window(EWindowType::Material, event_loop_window_target)?;
        let window = &*window_context.window.borrow();

        engine
            .set_new_window(
                window_context.get_id(),
                window,
                window_context.get_width(),
                window_context.get_height(),
                window.scale_factor() as f32,
            )
            .map_err(|err| anyhow!("{err}"))?;
        let viewport_id = egui::ViewportId::from_hash_of(window_context.get_id());

        let mut egui_winit_state = egui_winit::State::new(
            context.clone(),
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            None,
        );

        egui_winit_state.egui_input_mut().viewport_id = viewport_id;
        egui_winit_state.egui_input_mut().viewports =
            std::iter::once((viewport_id, Default::default())).collect();
        let material_view = MaterialView::new();
        let data_source = DataSource {
            current_open_material: None,
            is_shader_code_window_open: false,
        };
        Ok(MaterialUIWindow {
            egui_winit_state,
            material_view,
            data_source,
            context,
        })
    }

    pub fn window_event_process(
        &mut self,
        // context: &egui::Context,
        window_id: isize,
        window: &mut winit::window::Window,
        event: &WindowEvent,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
        engine: &mut Engine,
        window_manager: &mut WindowsManager,
    ) {
        let _ = self.egui_winit_state.on_window_event(window, event);
        let _ = event_loop_window_target;
        match event {
            WindowEvent::Resized(size) => {
                engine.resize(window_id, size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                window_manager.remove_window(EWindowType::Material);
                engine.remove_window(window_id);
            }
            WindowEvent::RedrawRequested => {
                engine.window_redraw_requested_begin(window_id);
                crate::ui::misc::ui_begin(&mut self.egui_winit_state, window);
                self.material_view.draw(
                    self.data_source.current_open_material.clone(),
                    &self.context,
                    &mut self.data_source,
                );
                let gui_render_output =
                    crate::ui::misc::ui_end(&mut self.egui_winit_state, window, window_id);
                engine.draw_gui(gui_render_output);
                window.request_redraw();
                engine.window_redraw_requested_end(window_id);
            }
            _ => {}
        }
    }
}
