use renderdoc::{CaptureOption, Error, RenderDoc, V141};
use std::collections::{HashMap, VecDeque};

pub struct Context {
    pub capture_commands: VecDeque<()>,
    inner: RenderDoc<V141>,
}

pub enum ECaptureOptionValue {
    U32(u32),
    Float(f32),
}

impl Context {
    pub fn new() -> crate::error::Result<Self> {
        let rd: Result<RenderDoc<V141>, Error> = RenderDoc::new();
        let rd = rd.map_err(|err| {
            crate::error::Error::RenderDoc(err, Some("Expect launched by RenderDoc".to_string()))
        })?;
        log::trace!("RenderDoc api version {:?}", rd.get_api_version());
        Ok(Self {
            capture_commands: VecDeque::new(),
            inner: rd,
        })
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
        unsafe { device.start_graphics_debugger_capture() };
        // use wgpu::hal::api::Dx12;
        // unsafe {
        //     let hal_device = device.as_hal::<Dx12>();
        //     if let Some(dev) = hal_device {
        //         let raw_device = dev.raw_device();
        //         let raw_ptr = raw_device
        //             as *const windows::Win32::Graphics::Direct3D12::ID3D12Device
        //             as *const std::ffi::c_void;
        //         let dev_ptr = DevicePointer::from(raw_ptr);
        //         self.inner.start_frame_capture(dev_ptr, std::ptr::null());
        //     } else {
        //         self.inner
        //             .start_frame_capture(std::ptr::null(), std::ptr::null());
        //     }
        // };
    }

    pub fn stop_capture(&mut self, device: &wgpu::Device) {
        // use wgpu::hal::api::Dx12;
        // unsafe {
        //     let hal_device = device.as_hal::<Dx12>();
        //     if let Some(dev) = hal_device {
        //         let raw_device = dev.raw_device();
        //         let raw_ptr = raw_device
        //             as *const windows::Win32::Graphics::Direct3D12::ID3D12Device
        //             as *const std::ffi::c_void;
        //         let dev_ptr = DevicePointer::from(raw_ptr);
        //         self.inner.end_frame_capture(dev_ptr, std::ptr::null());
        //     } else {
        //         self.inner
        //             .end_frame_capture(std::ptr::null(), std::ptr::null());
        //     }
        // };
        unsafe { device.stop_graphics_debugger_capture() };
    }
}
