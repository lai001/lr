use crate::error::Result;

pub struct WGPUContext {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
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
    ) -> wgpu::SurfaceConfiguration {
        let swapchain_capabilities = surface.get_capabilities(adapter);
        let mut swapchain_format = swapchain_capabilities.formats[0];
        for format in &swapchain_capabilities.formats {
            if format == &wgpu::TextureFormat::Rgba8UnormSrgb {
                swapchain_format = *format;
            }
        }
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: swapchain_format,
            width: surface_width,
            height: surface_height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        log::info!("surface_config {:#?}", surface_config);
        surface.configure(device, &surface_config);
        surface_config
    }

    pub fn new<W>(
        window: &W,
        surface_width: u32,
        surface_height: u32,
        power_preference: Option<wgpu::PowerPreference>,
        instance_desc: Option<wgpu::InstanceDescriptor>,
    ) -> Result<WGPUContext>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
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
                required_features: {
                    let mut features = wgpu::Features::default();
                    features.insert(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES);
                    features.insert(wgpu::Features::CLEAR_TEXTURE);
                    features.insert(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS);
                    features
                },
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))
        .map_err(|err| crate::error::Error::RequestDeviceError(err))?;

        let surface_config =
            Self::surface_configure(&surface, surface_width, surface_height, &adapter, &device);

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

        Ok(WGPUContext {
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_config,
        })
    }

    pub fn set_new_window<W>(
        &mut self,
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
            );
            surface.configure(&self.device, &surface_config);
            Ok(())
        } else {
            Err(crate::error::Error::SurfaceNotSupported)
        }
    }

    pub fn window_resized(&mut self, width: u32, height: u32) -> bool {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            true
        } else {
            false
        }
    }

    pub fn get_surface_capabilities(&self) -> wgpu::SurfaceCapabilities {
        self.surface.get_capabilities(&self.adapter)
    }

    pub fn get_current_swapchain_format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }

    pub fn get_current_surface_texture(
        &self,
    ) -> std::result::Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    pub fn get_device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn get_queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn get_surface_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.surface_config
    }
}
