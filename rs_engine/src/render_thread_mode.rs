use crate::error::Result;
use crate::thread_pool;
use rs_foundation::channel::SingleConsumeChnnel;
use rs_render::command::{RenderCommand, RenderOutput2};
use rs_render::renderer::Renderer;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

// #[derive(Clone)]
pub struct MultipleThreadRenderer {
    renderer: Arc<Mutex<Renderer>>,
    channel: Arc<SingleConsumeChnnel<RenderCommand, Option<RenderOutput2>>>,
    render_outputs: VecDeque<RenderOutput2>,
}

impl MultipleThreadRenderer {
    pub fn new(renderer: Renderer) -> Self {
        let renderer = Arc::new(Mutex::new(renderer));
        let channel = Self::spawn_render_thread(renderer.clone());
        Self {
            renderer,
            channel,
            render_outputs: VecDeque::new(),
        }
    }

    fn spawn_render_thread(
        renderer: Arc<Mutex<Renderer>>,
    ) -> Arc<SingleConsumeChnnel<RenderCommand, Option<RenderOutput2>>> {
        let channel =
            SingleConsumeChnnel::<RenderCommand, Option<RenderOutput2>>::shared(Some(2), None);
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

pub struct SingleThreadRenderer {
    renderer: Renderer,
    render_outputs: VecDeque<RenderOutput2>,
}

impl SingleThreadRenderer {
    pub fn new(renderer: Renderer) -> Self {
        Self {
            renderer,
            render_outputs: VecDeque::new(),
        }
    }
}

pub enum ERenderThreadMode {
    Single(SingleThreadRenderer),
    Multiple(MultipleThreadRenderer),
}

impl ERenderThreadMode {
    pub fn from(renderer: Renderer, is_multiple_thread: bool) -> ERenderThreadMode {
        if is_multiple_thread {
            ERenderThreadMode::Multiple(MultipleThreadRenderer::new(renderer))
        } else {
            ERenderThreadMode::Single(SingleThreadRenderer::new(renderer))
        }
    }

    pub fn send_command(&mut self, command: RenderCommand) {
        match self {
            ERenderThreadMode::Single(renderer) => {
                let output = renderer.renderer.send_command(command);
                if let Some(output) = output {
                    renderer.render_outputs.push_back(output);
                }
            }
            ERenderThreadMode::Multiple(renderer) => {
                renderer.channel.to_b(command);
            }
        }
    }

    pub fn set_new_window<W>(
        &mut self,
        window_id: isize,
        window: &W,
        surface_width: u32,
        surface_height: u32,
    ) -> Result<()>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        match self {
            ERenderThreadMode::Single(renderer) => Ok(renderer
                .renderer
                .set_new_window(window_id, window, surface_width, surface_height)
                .map_err(|err| crate::error::Error::RendererError(err))?),
            ERenderThreadMode::Multiple(renderer) => Ok(renderer
                .renderer
                .lock()
                .unwrap()
                .set_new_window(window_id, window, surface_width, surface_height)
                .map_err(|err| crate::error::Error::RendererError(err))?),
        }
    }

    pub fn recv_output(&mut self) {
        loop {
            match self {
                ERenderThreadMode::Single(_) => {
                    break;
                }
                ERenderThreadMode::Multiple(renderer) => {
                    let render_output = renderer.channel.from_b_try_recv();
                    match render_output {
                        Ok(render_output) => {
                            if let Some(render_output) = render_output {
                                renderer.render_outputs.push_back(render_output);
                            }
                        }
                        Err(err) => match err {
                            std::sync::mpsc::TryRecvError::Empty => {
                                break;
                            }
                            std::sync::mpsc::TryRecvError::Disconnected => {
                                panic!();
                            }
                        },
                    }
                }
            }
        }
    }
}

impl Drop for ERenderThreadMode {
    fn drop(&mut self) {
        match self {
            ERenderThreadMode::Single(_) => {}
            ERenderThreadMode::Multiple(renderer) => {
                renderer.channel.send_stop_signal_and_wait();
            }
        }
    }
}
