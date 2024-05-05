use std::collections::HashMap;

#[derive(Clone)]
pub struct EGUIRenderOutput {
    pub window_id: isize,
    pub textures_delta: egui::TexturesDelta,
    pub clipped_primitives: Vec<egui::ClippedPrimitive>,
}

pub struct EGUIRenderer {
    egui_wgpu_renderer: egui_wgpu::Renderer,
    screen_descriptors: HashMap<isize, egui_wgpu::ScreenDescriptor>,
}

impl EGUIRenderer {
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        msaa_samples: u32,
        screen_descriptors: HashMap<isize, egui_wgpu::ScreenDescriptor>,
    ) -> EGUIRenderer {
        let egui_wgpu_renderer =
            egui_wgpu::Renderer::new(device, output_format, None, msaa_samples);

        EGUIRenderer {
            egui_wgpu_renderer,
            screen_descriptors,
        }
    }

    pub fn get_screen_descriptors_mut(
        &mut self,
    ) -> &mut HashMap<isize, egui_wgpu::ScreenDescriptor> {
        &mut self.screen_descriptors
    }

    pub fn add_screen_descriptor(
        &mut self,
        window_id: isize,
        screen_descriptor: egui_wgpu::ScreenDescriptor,
    ) {
        self.screen_descriptors.insert(window_id, screen_descriptor);
    }

    pub fn remove_screen_descriptor(&mut self, window_id: isize) {
        self.screen_descriptors.remove(&window_id);
    }

    pub fn change_size(&mut self, window_id: isize, width: u32, height: u32) {
        if let Some(screen_descriptor) = self.screen_descriptors.get_mut(&window_id) {
            screen_descriptor.size_in_pixels[0] = width;
            screen_descriptor.size_in_pixels[1] = height;
        }
    }

    pub fn render(
        &mut self,
        gui_render_output: &EGUIRenderOutput,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        output_view: &wgpu::TextureView,
    ) {
        let EGUIRenderOutput {
            textures_delta,
            clipped_primitives,
            window_id,
        } = gui_render_output;
        for (id, image_delta) in &textures_delta.set {
            self.egui_wgpu_renderer
                .update_texture(&device, &queue, *id, image_delta);
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("EGUIRenderer.CommandEncoder"),
        });
        let Some(screen_descriptor) = self.screen_descriptors.get(&window_id) else {
            return;
        };
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
