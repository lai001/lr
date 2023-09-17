use crate::depth_texture::DepthTexture;
use wgpu::{InstanceDescriptor, SurfaceCapabilities, SurfaceTexture};

pub struct WGPUContext {
    pub instance: wgpu::Instance,
    surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    depth_texture: DepthTexture,
}

impl WGPUContext {
    fn new_surface(instance: &wgpu::Instance, window: &winit::window::Window) -> wgpu::Surface {
        unsafe { instance.create_surface(window) }.unwrap()
    }

    pub fn new(
        window: &winit::window::Window,
        power_preference: Option<wgpu::PowerPreference>,
        instance_desc: Option<InstanceDescriptor>,
    ) -> WGPUContext {
        // let instance = wgpu::Instance::default();
        let instance = wgpu::Instance::new(instance_desc.unwrap_or_default());
        let surface = Self::new_surface(&instance, window);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: power_preference.unwrap_or(wgpu::PowerPreference::default()),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: {
                    let mut features = wgpu::Features::default();
                    features.insert(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES);
                    features.insert(wgpu::Features::CLEAR_TEXTURE);
                    features.insert(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS);
                    features.insert(wgpu::Features::POLYGON_MODE_LINE);
                    features
                },
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))
        .unwrap();

        let window_size = window.inner_size();
        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let mut swapchain_format = swapchain_capabilities.formats[0];

        for format in &swapchain_capabilities.formats {
            if format == &wgpu::TextureFormat::Rgba8UnormSrgb {
                swapchain_format = *format;
            }
        }

        log::info!("swapchain_capabilities {:#?}", swapchain_capabilities);
        log::info!("adapter info: {:#?}", adapter.get_info());
        log::info!("adapter limits: {:#?}", adapter.limits());
        log::info!("adapter features: {:#?}", adapter.features());
        log::info!("swapchain_format: {:?}", swapchain_format);
        log::info!(
            "default SamplerDescriptor: {:?}",
            wgpu::SamplerDescriptor::default()
        );
        log::info!(
            "default TextureViewDescriptor: {:?}",
            wgpu::TextureViewDescriptor::default()
        );

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: swapchain_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);
        let depth_texture = DepthTexture::new(window_size.width, window_size.height, &device);

        WGPUContext {
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_config,
            depth_texture,
        }
    }

    pub fn window_resized(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.surface_config.width = size.width;
            self.surface_config.height = size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.depth_texture = DepthTexture::new(size.width, size.height, &self.device);
        }
    }

    pub fn get_surface_capabilities(&self) -> SurfaceCapabilities {
        self.surface.get_capabilities(&self.adapter)
    }

    pub fn get_current_swapchain_format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }

    pub fn get_depth_texture_view(&self) -> wgpu::TextureView {
        self.depth_texture.get_view()
    }

    pub fn get_current_surface_texture(&self) -> SurfaceTexture {
        self.surface.get_current_texture().unwrap()
    }
}
