use crate::error::Result;
use crate::shader_library::ShaderLibrary;
use crate::{egui_render::EGUIRenderer, wgpu_context::WGPUContext};
use std::collections::HashMap;

pub struct Renderer {
    wgpu_context: WGPUContext,
    egui_render_pass: EGUIRenderer,
    screen_descriptor: egui_wgpu_backend::ScreenDescriptor,
    shader_library: ShaderLibrary,
}

impl Renderer {
    pub fn from_context(
        gui_context: egui::Context,
        wgpu_context: WGPUContext,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
    ) -> Renderer {
        let egui_render_pass = EGUIRenderer::new(
            wgpu_context.get_device(),
            gui_context,
            wgpu_context.get_current_swapchain_format(),
            1,
        );
        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: surface_width,
            physical_height: surface_height,
            scale_factor,
        };
        let shader_library = ShaderLibrary::new();
        Renderer {
            wgpu_context,
            egui_render_pass,
            screen_descriptor,
            shader_library,
        }
    }

    pub fn from_window<W>(
        window: &W,
        gui_context: egui::Context,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
    ) -> Result<Renderer>
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        let wgpu_context = WGPUContext::new(
            window,
            surface_width,
            surface_height,
            Some(wgpu::PowerPreference::HighPerformance),
            Some(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
                flags: wgpu::InstanceFlags::default(),
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            }),
        );
        let wgpu_context = match wgpu_context {
            Ok(wgpu_context) => wgpu_context,
            Err(err) => return Err(err),
        };
        Ok(Self::from_context(
            gui_context,
            wgpu_context,
            surface_width,
            surface_height,
            scale_factor,
        ))
    }

    pub fn set_new_window<
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    >(
        &mut self,
        window: &W,
        surface_width: u32,
        surface_height: u32,
    ) -> Result<()> {
        self.wgpu_context
            .set_new_window(window, surface_width, surface_height)
    }

    pub fn present(&mut self, full_output: egui::FullOutput) {
        let texture = match self.wgpu_context.get_current_surface_texture() {
            Ok(texture) => texture,
            Err(err) => {
                if err != wgpu::SurfaceError::Outdated {
                    log::warn!("{}", err);
                }
                return;
            }
        };

        let output_view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let device = self.wgpu_context.get_device();
            let queue = self.wgpu_context.get_queue();

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

    pub fn load_shader<K>(&mut self, shaders: HashMap<K, String>)
    where
        K: ToString,
    {
        self.shader_library
            .load_shader_from(shaders, self.wgpu_context.get_device());
    }

    pub fn resize(&mut self, surface_width: u32, surface_height: u32) {
        self.screen_descriptor.physical_width = surface_width;
        self.screen_descriptor.physical_height = surface_height;
        self.wgpu_context
            .window_resized(surface_width, surface_height);
    }
}
