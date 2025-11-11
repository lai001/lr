use crate::error::Result;
use std::collections::HashMap;
use std::fmt::Debug;
#[cfg(feature = "wgpu26")]
use wgpu26 as wgpu;

pub struct WindowTarget<
    'a,
    W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
> {
    pub window: &'a W,
    pub surface_width: u32,
    pub surface_height: u32,
}

pub struct RSurface {
    pub window_id: isize,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
}

pub struct WGPUContext {
    instance: wgpu::Instance,
    surfaces: HashMap<isize, RSurface>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Debug for WGPUContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("WGPUContext");
        debug_struct
            .field("device", &self.device)
            .field("adapter info", &self.adapter.get_info())
            .field("adapter limits", &self.adapter.limits())
            .field(
                "default SamplerDescriptor",
                &wgpu::SamplerDescriptor::default(),
            )
            .field(
                "default TextureViewDescriptor",
                &wgpu::TextureViewDescriptor::default(),
            )
            .field(
                "default PrimitiveState",
                &wgpu::TextureViewDescriptor::default(),
            );
        for (window_id, surface) in self.surfaces.iter() {
            let surface_config = &surface.surface_config;
            let swapchain_capabilities = surface.surface.get_capabilities(&self.adapter);
            let swapchain_format = &surface_config.format;
            let guaranteed_format_features =
                swapchain_format.guaranteed_format_features(self.device.features());
            debug_struct.field("Window id", window_id);
            debug_struct.field("swapchain_capabilities", &swapchain_capabilities);
            debug_struct.field("using swapchain_format", &swapchain_format);
            debug_struct.field("guaranteed_format_features", &guaranteed_format_features);
            debug_struct.field("surface_config", &surface_config);
            debug_struct.field("surface_config", &surface_config);
        }
        debug_struct.finish()
    }
}

impl WGPUContext {
    fn new_surface<W>(instance: &wgpu::Instance, window: &W) -> Result<wgpu::Surface<'static>>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        unsafe {
            let surface_target = wgpu::SurfaceTargetUnsafe::from_window(window)
                .map_err(|err| crate::error::Error::WindowError(err))?;
            instance
                .create_surface_unsafe(surface_target)
                .map_err(|err| crate::error::Error::CreateSurfaceError(err))
        }
    }

    fn surface_configure(
        surface: &wgpu::Surface,
        surface_width: u32,
        surface_height: u32,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
    ) -> Result<wgpu::SurfaceConfiguration> {
        let swapchain_capabilities = surface.get_capabilities(adapter);
        let swapchain_format = *swapchain_capabilities
            .formats
            .iter()
            .find(|x| **x == wgpu::TextureFormat::Rgba8UnormSrgb)
            .unwrap_or(swapchain_capabilities.formats.first().ok_or(
                crate::error::Error::Other(Some(format!("No swapchain format supported"))),
            )?);
        let present_mode = *swapchain_capabilities
            .present_modes
            .iter()
            .find(|x| **x == wgpu::PresentMode::Fifo)
            .unwrap_or(swapchain_capabilities.present_modes.first().ok_or(
                crate::error::Error::Other(Some(format!("No present mode supported"))),
            )?);
        let alpha_mode = *swapchain_capabilities
            .alpha_modes
            .iter()
            .find(|x| **x == wgpu::CompositeAlphaMode::Inherit)
            .unwrap_or(swapchain_capabilities.alpha_modes.first().ok_or(
                crate::error::Error::Other(Some(format!("No alpha mode supported"))),
            )?);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: swapchain_capabilities.usages,
            format: swapchain_format,
            width: surface_width,
            height: surface_height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(device, &surface_config);
        Ok(surface_config)
    }

    fn adapter_device_queue(
        instance: &wgpu::Instance,
        compatible_surface: Option<&wgpu::Surface<'_>>,
        power_preference: Option<wgpu::PowerPreference>,
        device_debug_label: Option<&str>,
    ) -> crate::error::Result<(wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: power_preference.unwrap_or(wgpu::PowerPreference::default()),
            compatible_surface,
            force_fallback_adapter: false,
        }))
        .map_err(|err| crate::error::Error::RequestAdapterError(err))?;
        #[allow(unused_mut)]
        let mut required_features = adapter.features();
        #[cfg(feature = "wgpu_latest")]
        let experimental_features = {
            let experimental_features: wgpu::ExperimentalFeatures = Self::experimental_features();
            if !experimental_features.is_enabled() {
                required_features.remove(wgpu::Features::all_experimental_mask());
            }
            experimental_features
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features,
            required_limits: adapter.limits(),
            label: device_debug_label,
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
            #[cfg(feature = "wgpu_latest")]
            experimental_features,
        }))
        .map_err(|err| crate::error::Error::RequestDeviceError(err))?;
        Ok((adapter, device, queue))
    }

    pub fn new<W>(
        mut window_targets: HashMap<isize, WindowTarget<W>>,
        power_preference: Option<wgpu::PowerPreference>,
        instance_desc: Option<wgpu::InstanceDescriptor>,
        device_debug_label: Option<&str>,
    ) -> Result<WGPUContext>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        let _span = tracy_client::span!();
        let instance = wgpu::Instance::new(&instance_desc.unwrap_or_default());
        let mut wgpu_surfaces = Vec::with_capacity(window_targets.len());
        for (_, window_target) in window_targets.iter() {
            let surface = Self::new_surface(&instance, window_target.window)?;
            wgpu_surfaces.push(surface);
        }
        let (adapter, device, queue) = Self::adapter_device_queue(
            &instance,
            wgpu_surfaces.first(),
            power_preference,
            device_debug_label,
        )?;
        let mut surfaces: HashMap<isize, RSurface> = HashMap::with_capacity(window_targets.len());
        for (surface, (window_id, window_target)) in
            wgpu_surfaces.drain(..).zip(window_targets.drain())
        {
            let surface_config = Self::surface_configure(
                &surface,
                window_target.surface_width,
                window_target.surface_height,
                &adapter,
                &device,
            )?;
            surfaces.insert(
                window_id,
                RSurface {
                    window_id,
                    surface,
                    surface_config,
                },
            );
        }
        Ok(WGPUContext {
            instance,
            adapter,
            device,
            queue,
            surfaces,
        })
    }

    pub fn windowless(
        power_preference: Option<wgpu::PowerPreference>,
        instance_desc: Option<wgpu::InstanceDescriptor>,
        device_debug_label: Option<&str>,
    ) -> Result<WGPUContext> {
        let _span = tracy_client::span!();
        let instance = wgpu::Instance::new(&instance_desc.unwrap_or_default());
        let (adapter, device, queue) =
            Self::adapter_device_queue(&instance, None, power_preference, device_debug_label)?;
        Ok(WGPUContext {
            instance,
            adapter,
            device,
            queue,
            surfaces: HashMap::new(),
        })
    }

    #[cfg(feature = "wgpu_latest")]
    fn experimental_features() -> wgpu::ExperimentalFeatures {
        wgpu::ExperimentalFeatures::disabled()
    }

    pub fn get_device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn get_queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn remove_window(&mut self, window_id: isize) {
        self.surfaces.remove(&window_id);
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
        let surface = Self::new_surface(&self.instance, window)?;
        if self.adapter.is_surface_supported(&surface) {
            let surface_config = Self::surface_configure(
                &surface,
                surface_width,
                surface_height,
                &self.adapter,
                &self.device,
            )?;
            let surface_texture_formats = self.surfaces.values().map(|x| &x.surface_config.format);
            for format in surface_texture_formats {
                if format != &surface_config.format {
                    return Err(crate::error::Error::SurfaceNotSupported);
                }
            }
            surface.configure(&self.device, &surface_config);
            self.surfaces.insert(
                window_id,
                RSurface {
                    window_id,
                    surface,
                    surface_config,
                },
            );
            Ok(())
        } else {
            Err(crate::error::Error::SurfaceNotSupported)
        }
    }

    pub fn window_resized(&mut self, window_id: isize, width: u32, height: u32) -> bool {
        if width <= 0 || height <= 0 {
            return false;
        }
        if let Some(surface) = self.surfaces.get_mut(&window_id) {
            surface.surface_config.width = width;
            surface.surface_config.height = height;
            surface
                .surface
                .configure(&self.device, &surface.surface_config);
            true
        } else {
            false
        }
    }

    pub fn get_surface_capabilities(&self, window_id: isize) -> wgpu::SurfaceCapabilities {
        let surface = self.surfaces.get(&window_id).expect("Not null");
        surface.surface.get_capabilities(&self.adapter)
    }

    pub fn get_current_swapchain_format(&self, window_id: isize) -> wgpu::TextureFormat {
        let surface = self.surfaces.get(&window_id).expect("Not null");
        surface.surface_config.format
    }

    pub fn get_current_surface_texture(
        &self,
        window_id: isize,
    ) -> std::result::Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        let surface = self.surfaces.get(&window_id).expect("Not null");
        surface.surface.get_current_texture()
    }

    pub fn get_surface_config(&self, window_id: isize) -> &wgpu::SurfaceConfiguration {
        let surface = self.surfaces.get(&window_id).expect("Not null");
        &surface.surface_config
    }

    pub fn get_window_ids(&self) -> Vec<isize> {
        self.surfaces.keys().map(|x| *x).collect()
    }
}

#[cfg(test)]
mod test {
    use crate::wgpu_context::WGPUContext;

    #[test]
    fn test() {
        let context = WGPUContext::windowless(None, None, None).unwrap();
        println!("{:#?}", context);
    }
}
