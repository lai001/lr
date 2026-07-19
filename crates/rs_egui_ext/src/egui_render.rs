use egui_wgpu::RendererOptions;
use wgpu::{Color, Operations};

#[derive(Clone)]
pub struct EGUIRenderOutput {
    pub pixels_per_point: f32,
    pub textures_delta: egui::TexturesDelta,
    pub clipped_primitives: Vec<egui::ClippedPrimitive>,
}

pub struct EGUIRenderer {
    egui_wgpu_renderer: egui_wgpu::Renderer,
}

impl EGUIRenderer {
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        renderer_options: RendererOptions,
    ) -> EGUIRenderer {
        let egui_wgpu_renderer = egui_wgpu::Renderer::new(device, output_format, renderer_options);
        EGUIRenderer { egui_wgpu_renderer }
    }

    pub fn render(
        &mut self,
        ops: Operations<Color>,
        gui_render_output: &EGUIRenderOutput,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        output_view: &wgpu::TextureView,
    ) {
        let EGUIRenderOutput {
            textures_delta,
            clipped_primitives,
            pixels_per_point,
        } = gui_render_output;
        for (id, image_delta) in &textures_delta.set {
            self.egui_wgpu_renderer
                .update_texture(&device, &queue, *id, image_delta);
        }
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                output_view.texture().width(),
                output_view.texture().height(),
            ],
            pixels_per_point: *pixels_per_point,
        };
        if clipped_primitives.is_empty() {
            return;
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
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_render"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops,
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            // TODO!
            let mut render_pass = render_pass.forget_lifetime();
            self.egui_wgpu_renderer.render(
                &mut render_pass,
                &clipped_primitives,
                &screen_descriptor,
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
