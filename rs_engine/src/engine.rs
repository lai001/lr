use std::collections::HashMap;

use crate::error::Result;
use crate::{
    error,
    logger::{Logger, LoggerConfiguration},
    resource_manager::ResourceManager,
};
use egui::RawInput;
use rs_artifact::{artifact::ArtifactReader, build_asset_url, resource_type::EResourceType};
use rs_render::renderer::Renderer;

pub struct Engine {
    renderer: Renderer,
    resource_manager: ResourceManager,
    logger: Logger,
}

impl Engine {
    pub fn new<W>(
        window: &W,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
        artifact_reader: Option<ArtifactReader>,
    ) -> Result<Engine>
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        let logger = Logger::new(LoggerConfiguration {
            is_write_to_file: true,
        });

        let renderer = rs_render::renderer::Renderer::from_window(
            window,
            surface_width,
            surface_height,
            scale_factor,
        );
        let mut renderer = match renderer {
            Ok(renderer) => renderer,
            Err(err) => return Err(crate::error::Error::RendererError(err)),
        };
        let mut resource_manager = ResourceManager::default();
        resource_manager.set_artifact_reader(artifact_reader);
        let mut shaders: HashMap<String, String> = HashMap::new();

        for shader_source_code in resource_manager.get_all_shader_source_codes() {
            shaders.insert(shader_source_code.url.to_string(), shader_source_code.code);
        }

        renderer.load_shader(shaders);
        let engine = Engine {
            logger,
            renderer,
            resource_manager,
        };

        Ok(engine)
    }

    pub fn redraw(&mut self, raw_input: &RawInput) {
        self.renderer.present(raw_input.clone());
    }

    pub fn set_new_window<W>(
        &mut self,
        window: &W,
        surface_width: u32,
        surface_height: u32,
    ) -> Result<()>
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        let result = self
            .renderer
            .set_new_window(window, surface_width, surface_height);
        match result {
            Ok(_) => Ok(()),
            Err(err) => return Err(crate::error::Error::RendererError(err)),
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.logger.flush();
    }
}
