use crate::error::Result;
use std::collections::HashMap;

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
        log::info!("swapchain_capabilities {:#?}", swapchain_capabilities);

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

        let guaranteed_format_features =
            swapchain_format.guaranteed_format_features(device.features());
        log::info!(
            "using swapchain_format: {:?}, guaranteed_format_features {:#?}",
            swapchain_format,
            guaranteed_format_features
        );
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
        log::info!("surface_config {:#?}", surface_config);
        surface.configure(device, &surface_config);
        Ok(surface_config)
    }

    pub fn new<W>(
        window_id: isize,
        window: &W,
        surface_width: u32,
        surface_height: u32,
        power_preference: Option<wgpu::PowerPreference>,
        instance_desc: Option<wgpu::InstanceDescriptor>,
    ) -> Result<WGPUContext>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        let _span = tracy_client::span!();

        let instance = wgpu::Instance::new(instance_desc.unwrap_or_default());
        let surface = Self::new_surface(&instance, window)?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: power_preference.unwrap_or(wgpu::PowerPreference::default()),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or(crate::error::Error::RequestAdapterFailed)?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: adapter.features(),
                required_limits: adapter.limits(),
                label: Some("Engine"),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        ))
        .map_err(|err| crate::error::Error::RequestDeviceError(err))?;

        let surface_config =
            Self::surface_configure(&surface, surface_width, surface_height, &adapter, &device)?;

        Self::dump(&adapter);

        Ok(WGPUContext {
            instance,
            adapter,
            device,
            queue,
            surfaces: HashMap::from([(
                window_id,
                RSurface {
                    window_id,
                    surface,
                    surface_config,
                },
            )]),
        })
    }

    pub fn windowless(
        power_preference: Option<wgpu::PowerPreference>,
        instance_desc: Option<wgpu::InstanceDescriptor>,
    ) -> Result<WGPUContext> {
        let _span = tracy_client::span!();

        let instance = wgpu::Instance::new(instance_desc.unwrap_or_default());

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: power_preference.unwrap_or(wgpu::PowerPreference::default()),
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok_or(crate::error::Error::RequestAdapterFailed)?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: adapter.features(),
                required_limits: adapter.limits(),
                label: Some("Engine"),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        ))
        .map_err(|err| crate::error::Error::RequestDeviceError(err))?;

        Self::dump(&adapter);

        Ok(WGPUContext {
            instance,
            adapter,
            device,
            queue,
            surfaces: HashMap::new(),
        })
    }

    fn dump(adapter: &wgpu::Adapter) {
        log::info!("adapter info: {:#?}", adapter.get_info());
        log::info!("adapter limits: {:#?}", adapter.limits());
        log::info!("adapter features: {:#?}", adapter.features());
        log::info!(
            "default SamplerDescriptor: {:?}",
            wgpu::SamplerDescriptor::default()
        );
        log::info!(
            "default TextureViewDescriptor: {:?}",
            wgpu::TextureViewDescriptor::default()
        );
        log::info!(
            "default PrimitiveState: {:?}",
            wgpu::PrimitiveState::default()
        );
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
