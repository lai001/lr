pub struct EGUIRenderer {
    egui_wgpu_renderer: egui_wgpu::Renderer,
}

#[derive(Clone)]
pub struct EGUIRenderOutput {
    pub textures_delta: egui::TexturesDelta,
    pub clipped_primitives: Vec<egui::ClippedPrimitive>,
}

impl EGUIRenderer {
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> EGUIRenderer {
        let egui_wgpu_renderer =
            egui_wgpu::Renderer::new(device, output_format, None, msaa_samples);

        EGUIRenderer { egui_wgpu_renderer }
    }

    pub fn render(
        &mut self,
        gui_render_output: EGUIRenderOutput,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        output_view: &wgpu::TextureView,
    ) {
        let EGUIRenderOutput {
            textures_delta,
            clipped_primitives,
        } = gui_render_output;
        for (id, image_delta) in &textures_delta.set {
            self.egui_wgpu_renderer
                .update_texture(&device, &queue, *id, image_delta);
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("EGUIRenderer.CommandEncoder"),
        });

        let mut command_buffers = self.egui_wgpu_renderer.update_buffers(
            &device,
            &queue,
            &mut encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_render"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.egui_wgpu_renderer.render(
                &mut render_pass,
                &clipped_primitives,
                screen_descriptor,
            );
        }
        command_buffers.push(encoder.finish());
        queue.submit(command_buffers);
    }

    pub fn remove_texture_ids(&mut self, texture_ids: &[egui::TextureId]) {
        let textures: egui::TexturesDelta = egui::TexturesDelta {
            set: vec![],
            free: texture_ids.to_vec(),
        };
        for id in &textures.free {
            self.egui_wgpu_renderer.free_texture(id);
        }
    }

    pub fn create_image2(
        &mut self,
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        texture_filter: Option<wgpu::FilterMode>,
    ) -> egui::TextureId {
        self.egui_wgpu_renderer.register_native_texture(
            device,
            texture_view,
            texture_filter.unwrap_or(wgpu::FilterMode::Linear),
        )
    }
}
