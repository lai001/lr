use renderdoc::{CaptureOption, Error, RenderDoc, V110};
use std::collections::{HashMap, VecDeque};

pub struct Context {
    pub capture_commands: VecDeque<()>,
    inner: RenderDoc<V110>,
}

pub enum ECaptureOptionValue {
    U32(u32),
    Float(f32),
}

impl Context {
    pub fn new() -> Option<Self> {
        let rd: Result<RenderDoc<V110>, Error> = RenderDoc::new();
        match rd {
            Ok(rd) => {
                log::trace!("RenderDoc api version {:?}", rd.get_api_version());
                Some(Self {
                    capture_commands: VecDeque::new(),
                    inner: rd,
                })
            }
            Err(err) => {
                log::warn!("{}. {}", err, "Expect launched by RenderDoc");
                return None;
            }
        }
    }

    pub fn set_capture_option(
        &mut self,
        capture_options: HashMap<CaptureOption, ECaptureOptionValue>,
    ) {
        for (capture_option, value) in capture_options {
            match value {
                ECaptureOptionValue::U32(value) => {
                    self.inner.set_capture_option_u32(capture_option, value);
                }
                ECaptureOptionValue::Float(value) => {
                    self.inner.set_capture_option_f32(capture_option, value);
                }
            }
        }
    }

    pub fn start_capture(&mut self, device: &wgpu::Device) {
        device.start_capture();
        self.inner.trigger_capture();
    }

    pub fn stop_capture(&mut self, device: &wgpu::Device) {
        device.stop_capture();
        self.inner
            .end_frame_capture(std::ptr::null(), std::ptr::null());
    }
}
