use crate::error::Result;
use crate::{
    logger::{Logger, LoggerConfiguration},
    resource_manager::ResourceManager,
};
use rs_artifact::artifact::ArtifactReader;
use rs_render::renderer::Renderer;
use std::collections::HashMap;

pub struct Engine {
    renderer: Renderer,
    resource_manager: ResourceManager,
    logger: Logger,
    gui_context: egui::Context,
}

impl Engine {
    pub fn new<W>(
        window: &W,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
        gui_context: egui::Context,
        artifact_reader: Option<ArtifactReader>,
    ) -> Result<Engine>
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        let logger = Logger::new(LoggerConfiguration {
            is_write_to_file: true,
        });

        let renderer = Renderer::from_window(
            window,
            gui_context.clone(),
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
            renderer,
            resource_manager,
            logger,
            gui_context,
        };

        Ok(engine)
    }

    pub fn redraw(&mut self, full_output: egui::FullOutput) {
        self.renderer.present(full_output);
    }

    pub fn resize(&mut self, surface_width: u32, surface_height: u32) {
        self.renderer.resize(surface_width, surface_height);
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

    pub fn get_gui_context(&self) -> egui::Context {
        self.gui_context.clone()
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.logger.flush();
    }
}
