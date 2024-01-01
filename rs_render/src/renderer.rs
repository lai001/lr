use crate::{egui_render::EGUIRenderer, wgpu_context::WGPUContext};

pub struct Renderer {
    wgpu_context: WGPUContext,
    context: egui::Context,
    egui_render_pass: EGUIRenderer,
    screen_descriptor: egui_wgpu_backend::ScreenDescriptor,
}

impl Renderer {
    pub fn new(
        wgpu_context: WGPUContext,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
    ) -> Renderer {
        let context = egui::Context::default();
        context.set_fonts(egui::FontDefinitions::default());
        context.set_style(egui::Style::default());

        let egui_render_pass = EGUIRenderer::new(
            wgpu_context.get_device(),
            context.clone(),
            wgpu_context.get_current_swapchain_format(),
            1,
        );
        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: surface_width,
            physical_height: surface_height,
            scale_factor,
        };

        Renderer {
            wgpu_context,
            egui_render_pass,
            context,
            screen_descriptor,
        }
    }

    pub fn set_new_window<
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    >(
        &mut self,
        window: &W,
        surface_width: u32,
        surface_height: u32,
    ) -> bool {
        self.wgpu_context
            .set_new_window(window, surface_width, surface_height)
    }

    pub fn present(&mut self, raw_input: egui::RawInput) {
        let texture = self.wgpu_context.get_current_surface_texture();
        if let Err(error) = texture {
            log::error!("{}", error);
            panic!()
        }

        let texture = texture.unwrap();

        let output_view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let device = self.wgpu_context.get_device();
            let queue = self.wgpu_context.get_queue();

            self.context.begin_frame(raw_input);

            egui::Window::new("Pannel")
                .default_pos((200.0, 200.0))
                .show(&self.context, |ui| {
                    let response = ui.button("Button");
                    if response.clicked() {}
                    if ui.button("Button2").clicked() {}
                    ui.label(format!("Time: {:.2}", 0.0f32));
                });

            let full_output = self.context.end_frame();
            self.egui_render_pass.render(
                &full_output,
                queue,
                device,
                &self.screen_descriptor,
                &output_view,
            )
        }

        texture.present();
    }
}
