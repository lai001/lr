use super::material_view::MaterialView;
use crate::{custom_event::ECustomEventType, editor::WindowsManager, editor_context::EWindowType};
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
                let gui_render_output = (|| {
                    let egui_winit_state = &mut self.egui_winit_state;
                    {
                        let ctx = egui_winit_state.egui_ctx().clone();
                        let viewport_id = egui_winit_state.egui_input().viewport_id;
                        let viewport_info: &mut egui::ViewportInfo = egui_winit_state
                            .egui_input_mut()
                            .viewports
                            .get_mut(&viewport_id)
                            .unwrap();
                        egui_winit::update_viewport_info(viewport_info, &ctx, window);
                    }

                    let new_input = egui_winit_state.take_egui_input(window);

                    egui_winit_state.egui_ctx().begin_frame(new_input);

                    self.material_view.draw(
                        self.data_source.current_open_material.clone(),
                        &self.context,
                        &mut self.data_source,
                    );

                    egui_winit_state.egui_ctx().clear_animations();

                    let full_output = egui_winit_state.egui_ctx().end_frame();

                    egui_winit_state
                        .handle_platform_output(window, full_output.platform_output.clone());

                    let gui_render_output = rs_render::egui_render::EGUIRenderOutput {
                        textures_delta: full_output.textures_delta,
                        clipped_primitives: egui_winit_state
                            .egui_ctx()
                            .tessellate(full_output.shapes, full_output.pixels_per_point),
                        window_id,
                    };
                    Some(gui_render_output)
                })();

                if let Some(gui_render_output) = gui_render_output {
                    engine.redraw(gui_render_output);
                    engine.present(window_id);
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
