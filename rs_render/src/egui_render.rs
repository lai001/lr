pub struct EGUIRenderer {
    context: egui::Context,
    egui_render_pass: egui_wgpu_backend::RenderPass,
}

impl EGUIRenderer {
    pub fn new(
        device: &wgpu::Device,
        context: egui::Context,
        output_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> EGUIRenderer {
        let egui_render_pass =
            egui_wgpu_backend::RenderPass::new(&device, output_format, msaa_samples);
        EGUIRenderer {
            context,
            egui_render_pass,
        }
    }

    pub fn render(
        &mut self,
        full_output: &egui::FullOutput,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        screen_descriptor: &egui_wgpu_backend::ScreenDescriptor,
        output_view: &wgpu::TextureView,
    ) {
        let paint_jobs = self.context.tessellate(full_output.shapes.clone());
        let textures_delta: egui::TexturesDelta = full_output.textures_delta.clone();
        if let Err(error) = self
            .egui_render_pass
            .add_textures(&device, &queue, &textures_delta)
        {
            log::warn!("{error}");
            return;
        }
        self.egui_render_pass
            .update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("EGUIRenderer.CommandEncoder"),
        });
        if let Err(error) = self.egui_render_pass.execute(
            &mut encoder,
            &output_view,
            &paint_jobs,
            &screen_descriptor,
            None,
        ) {
            log::warn!("{error}");
            return;
        }
        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn remove_texture_ids(&mut self, texture_ids: &[egui::TextureId]) {
        let textures: egui::TexturesDelta = egui::TexturesDelta {
            set: vec![],
            free: texture_ids.to_vec(),
        };
        match self.egui_render_pass.remove_textures(textures.clone()) {
            Ok(_) => {}
            Err(error) => log::warn!("{error}"),
        }
    }

    pub fn create_image2(
        &mut self,
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        texture_filter: Option<wgpu::FilterMode>,
    ) -> egui::TextureId {
        self.egui_render_pass.egui_texture_from_wgpu_texture(
            device,
            texture_view,
            texture_filter.unwrap_or(wgpu::FilterMode::Linear),
        )
    }
}
