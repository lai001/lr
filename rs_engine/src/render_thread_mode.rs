use crate::thread_pool;
use rs_foundation::channel::SingleConsumeChnnel;
use rs_render::command::{RenderCommand, RenderOutput};
use rs_render::renderer::Renderer;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MultipleThreadRenderer {
    pub renderer: Arc<Mutex<Renderer>>,
    pub channel: Arc<SingleConsumeChnnel<RenderCommand, Option<RenderOutput>>>,
}

impl MultipleThreadRenderer {
    pub fn new(renderer: Renderer) -> Self {
        let renderer = Arc::new(Mutex::new(renderer));
        let channel = Self::spawn_render_thread(renderer.clone());
        Self { renderer, channel }
    }

    fn spawn_render_thread(
        renderer: Arc<Mutex<Renderer>>,
    ) -> Arc<SingleConsumeChnnel<RenderCommand, Option<RenderOutput>>> {
        let channel =
            SingleConsumeChnnel::<RenderCommand, Option<RenderOutput>>::shared(Some(2), None);
        thread_pool::ThreadPool::render().spawn({
            let renderer = renderer.clone();
            let channel = channel.clone();
            move || {
                channel.from_a_block_current_thread(|command| {
                    let mut renderer = renderer.lock().unwrap();
                    let output = renderer.send_command(command);
                    channel.to_a(output);
                });
            }
        });
        return channel;
    }
}

pub enum ERenderThreadMode {
    Single(Renderer),
    Multiple(MultipleThreadRenderer),
}

impl ERenderThreadMode {
    pub fn send_command(&mut self, command: RenderCommand) {
        match self {
            ERenderThreadMode::Single(renderer) => {
                renderer.send_command(command);
            }
            ERenderThreadMode::Multiple(renderer) => {
                renderer.channel.to_b(command);
            }
        }
    }
}
