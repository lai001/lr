use crate::{editor::Editor, editor_context::EWindowType};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use std::collections::HashMap;

pub struct WindowContext {
    pub window_type: EWindowType,
    pub window: SingleThreadMutType<winit::window::Window>,
}

impl WindowContext {
    pub fn get_id(&self) -> isize {
        u64::from(self.window.borrow().id()) as isize
    }

    pub fn get_width(&self) -> u32 {
        self.window.borrow().inner_size().width
    }

    pub fn get_height(&self) -> u32 {
        self.window.borrow().inner_size().height
    }
}

pub struct WindowsManager {
    pub window_contexts: HashMap<EWindowType, WindowContext>,
}

impl WindowsManager {
    pub fn new() -> WindowsManager {
        WindowsManager {
            window_contexts: HashMap::new(),
        }
    }

    pub fn add_new_window(&mut self, window_type: EWindowType, window: winit::window::Window) {
        self.window_contexts.insert(
            window_type,
            WindowContext {
                window_type,
                window: SingleThreadMut::new(window),
            },
        );
    }

    pub fn get_window_context_by_id(&self, window_id: isize) -> Option<&WindowContext> {
        let context = self
            .window_contexts
            .iter()
            .find(|x| x.1.get_id() == window_id);
        match context {
            Some(context) => Some(context.1),
            None => None,
        }
    }

    pub fn get_window_by_id(
        &self,
        window_id: isize,
    ) -> Option<SingleThreadMutType<winit::window::Window>> {
        let context = self
            .window_contexts
            .iter()
            .find(|x| x.1.get_id() == window_id);
        match context {
            Some(context) => Some(context.1.window.clone()),
            None => None,
        }
    }

    pub fn get_window_type_by_id(&self, window_id: isize) -> Option<EWindowType> {
        let context = self
            .window_contexts
            .iter()
            .find(|x| x.1.get_id() == window_id);
        match context {
            Some(context) => Some(context.1.window_type),
            None => None,
        }
    }

    pub fn get_main_window(&self) -> SingleThreadMutType<winit::window::Window> {
        self.window_contexts
            .get(&EWindowType::Main)
            .expect("Not null")
            .window
            .clone()
    }

    pub fn spwan_new_window(
        &mut self,
        window_type: EWindowType,
        active_event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> anyhow::Result<&mut WindowContext> {
        let scale_factor = active_event_loop
            .primary_monitor()
            .map(|x| x.scale_factor())
            .unwrap_or(1.0);
        let window_width = (1280 as f64 * scale_factor) as u32;
        let window_height = (720 as f64 * scale_factor) as u32;

        let window_attributes = winit::window::Window::default_attributes()
            .with_window_icon(Some(Editor::default_icon()?))
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .with_title(format!("{:?}", window_type))
            .with_inner_size(winit::dpi::PhysicalSize {
                width: window_width,
                height: window_height,
            });
        let child_window = active_event_loop.create_window(window_attributes)?;
        child_window.set_ime_allowed(true);

        let child_window = SingleThreadMut::new(child_window);
        self.window_contexts.insert(
            window_type,
            WindowContext {
                window_type,
                window: child_window,
            },
        );

        Ok(self
            .window_contexts
            .get_mut(&window_type)
            .ok_or(anyhow::anyhow!(""))?)
    }

    pub fn remove_window(&mut self, window_type: EWindowType) {
        self.window_contexts.remove(&window_type);
    }
}
