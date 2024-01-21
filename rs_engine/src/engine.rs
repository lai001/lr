use crate::error::Result;
use crate::thread_pool;
use crate::{
    logger::{Logger, LoggerConfiguration},
    resource_manager::ResourceManager,
};
use rs_artifact::artifact::ArtifactReader;
use rs_foundation::channel::SingleConsumeChnnel;
use rs_render::command::{
    BufferCreateInfo, CreateBuffer, DrawObject, EMaterialType, RenderCommand, RenderOutput,
};
use rs_render::renderer::Renderer;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Engine {
    renderer: Arc<Mutex<Renderer>>,
    channel: Arc<SingleConsumeChnnel<RenderCommand, Option<RenderOutput>>>,
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
        let renderer = match renderer {
            Ok(renderer) => renderer,
            Err(err) => return Err(crate::error::Error::RendererError(err)),
        };
        let renderer = Arc::new(Mutex::new(renderer));

        let mut resource_manager = ResourceManager::default();
        resource_manager.set_artifact_reader(artifact_reader);
        let mut shaders: HashMap<String, String> = HashMap::new();

        for shader_source_code in resource_manager.get_all_shader_source_codes() {
            shaders.insert(shader_source_code.url.to_string(), shader_source_code.code);
        }
        let channel = Self::spawn_render_thread(renderer.clone(), shaders);
        let engine = Engine {
            renderer,
            resource_manager,
            logger,
            gui_context,
            channel,
        };

        Ok(engine)
    }

    fn spawn_render_thread(
        renderer: Arc<Mutex<Renderer>>,
        shaders: HashMap<String, String>,
    ) -> Arc<SingleConsumeChnnel<RenderCommand, Option<RenderOutput>>> {
        let channel =
            SingleConsumeChnnel::<RenderCommand, Option<RenderOutput>>::shared(Some(2), None);
        thread_pool::ThreadPool::render().spawn({
            let renderer = renderer.clone();
            let shaders = shaders.clone();
            let channel = channel.clone();

            move || {
                {
                    let mut renderer = renderer.lock().unwrap();
                    renderer.load_shader(shaders);
                }

                channel.from_a_block_current_thread(|command| {
                    let mut renderer = renderer.lock().unwrap();
                    let output = renderer.send_command(command);
                    channel.to_a(output);
                });
            }
        });
        return channel;
    }

    pub fn redraw(&mut self, full_output: egui::FullOutput) {
        loop {
            match self.channel.from_b_try_recv() {
                Ok(_) => {}
                Err(_) => break,
            }
        }
        self.channel.to_b(RenderCommand::UiOutput(full_output));
        self.channel.to_b(RenderCommand::Present);
    }

    pub fn resize(&mut self, surface_width: u32, surface_height: u32) {
        self.channel
            .to_b(RenderCommand::Resize(rs_render::command::ResizeInfo {
                width: surface_width,
                height: surface_height,
            }));
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
        let result =
            self.renderer
                .lock()
                .unwrap()
                .set_new_window(window, surface_width, surface_height);
        match result {
            Ok(_) => Ok(()),
            Err(err) => return Err(crate::error::Error::RendererError(err)),
        }
    }

    pub fn get_gui_context(&self) -> egui::Context {
        self.gui_context.clone()
    }

    pub fn create_draw_object_from_static_mesh(
        &mut self,
        vertexes: &[rs_artifact::mesh_vertex::MeshVertex],
        indexes: &[u32],
        material_type: EMaterialType,
    ) -> DrawObject {
        let index_buffer_handle = self.resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some("StaticMesh::IndexBuffer".to_string()),
            contents: rs_foundation::cast_to_raw_buffer(&indexes).to_vec(),
            usage: wgpu::BufferUsages::INDEX,
        };
        let create_buffer = CreateBuffer {
            handle: *index_buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        self.channel.to_b(message);

        let vertex_buffer_handle = self.resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some("StaticMesh::VertexBuffer".to_string()),
            contents: rs_foundation::cast_to_raw_buffer(&vertexes).to_vec(),
            usage: wgpu::BufferUsages::VERTEX,
        };
        let create_buffer = CreateBuffer {
            handle: *vertex_buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        self.channel.to_b(message);

        let draw_object = DrawObject {
            vertex_buffers: vec![*vertex_buffer_handle],
            vertex_count: vertexes.len() as u32,
            index_buffer: Some(*index_buffer_handle),
            index_count: Some(indexes.len() as u32),
            material_type,
        };
        draw_object
    }

    pub fn draw(&mut self, draw_object: DrawObject) {
        self.channel.to_b(RenderCommand::DrawObject(draw_object));
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.channel.send_stop_signal_and_wait();
        self.logger.flush();
    }
}
