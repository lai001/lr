use super::{material_view::MaterialView, ui_window::UIWindow};
use crate::{
    content_folder::ContentFolder, editor_context::EWindowType, windows_manager::WindowsManager,
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
    pub folder: SingleThreadMutType<ContentFolder>,
    window_id: isize,
}

impl MaterialUIWindow {
    pub fn new(
        context: egui::Context,
        window_manager: &mut WindowsManager,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        engine: &mut Engine,
        folder: SingleThreadMutType<ContentFolder>,
    ) -> anyhow::Result<MaterialUIWindow> {
        let window_context = window_manager.spwan_new_window(
            EWindowType::Material,
            event_loop_window_target,
            None,
        )?;
        let window = &*window_context.window.borrow();
        let window_id = window_context.get_id();

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
            None,
        );

        egui_winit_state.egui_input_mut().viewport_id = viewport_id;
        egui_winit_state.egui_input_mut().viewports =
            std::iter::once((viewport_id, Default::default())).collect();
        let material_view = MaterialView::new(folder.clone());
        let data_source = DataSource {
            current_open_material: None,
            is_shader_code_window_open: false,
        };
        Ok(MaterialUIWindow {
            egui_winit_state,
            material_view,
            data_source,
            context,
            folder,
            window_id,
        })
    }
}

impl UIWindow for MaterialUIWindow {
    fn on_device_event(&mut self, device_event: &winit::event::DeviceEvent) {
        let _ = device_event;
    }

    fn on_window_event(
        &mut self,
        window_id: isize,
        window: &mut winit::window::Window,
        event: &WindowEvent,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        engine: &mut Engine,
        window_manager: &mut WindowsManager,
        is_request_close: &mut bool,
    ) {
        let _ = window_manager;
        let _ = is_request_close;
        let _ = self.egui_winit_state.on_window_event(window, event);
        let _ = event_loop_window_target;
        match event {
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

    fn get_window_id(&self) -> isize {
        self.window_id
    }
}
